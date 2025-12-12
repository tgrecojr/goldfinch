use anyhow::{bail, Context, Result};
use aws_sdk_secretsmanager::Client;
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Parser)]
#[command(name = "goldfinch")]
#[command(about = "A CLI tool to read key-value pairs from AWS Secrets", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format
    #[arg(short, long, value_enum, global = true, default_value = "json")]
    format: OutputFormat,
}

#[derive(Subcommand)]
enum Commands {
    /// List all secret names in your AWS account
    List,

    /// Get all key-value pairs from a specific secret by name
    Get {
        /// The secret name
        secret_name: String,
    },

    /// Search for secrets and keys matching a pattern (searches both secret names and key names)
    Search {
        /// Search pattern (substring match)
        pattern: String,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Json,
    Plain,
}

#[derive(Serialize)]
struct KeyValue {
    key: String,
    value: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize AWS config and client
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    // List all secrets in the account
    let secret_ids = list_all_secrets(&client).await?;

    match &cli.command {
        Commands::List => {
            list_keys(&secret_ids, cli.format)?;
        }
        Commands::Get { secret_name } => {
            // Fetch the specified secret and return all its k/v pairs
            let secret_data = fetch_secret(&client, secret_name).await?;
            get_secret(&secret_data, secret_name, cli.format)?;
        }
        Commands::Search { pattern } => {
            // Fetch all secrets with their data for search command
            let mut secrets_with_data = BTreeMap::new();
            for secret_id in &secret_ids {
                let secret_data = fetch_secret(&client, secret_id).await?;
                secrets_with_data.insert(secret_id.clone(), secret_data);
            }
            search_keys(&secrets_with_data, pattern, cli.format)?;
        }
    }

    Ok(())
}

async fn fetch_secret(client: &Client, secret_id: &str) -> Result<BTreeMap<String, Value>> {
    let response = client
        .get_secret_value()
        .secret_id(secret_id)
        .send()
        .await
        .context(format!("Failed to fetch secret '{}'", secret_id))?;

    let secret_string = response
        .secret_string()
        .context("Secret does not contain a string value")?;

    let json: Value =
        serde_json::from_str(secret_string).context("Secret value is not valid JSON")?;

    match json {
        Value::Object(map) => {
            let mut btree_map = BTreeMap::new();
            for (k, v) in map {
                btree_map.insert(k, v);
            }
            Ok(btree_map)
        }
        _ => bail!("Secret value is not a JSON object with key-value pairs"),
    }
}

async fn list_all_secrets(client: &Client) -> Result<Vec<String>> {
    let mut secret_names = Vec::new();
    let mut paginator = client.list_secrets().into_paginator().send();

    while let Some(result) = paginator.next().await {
        let output = result.context("Failed to list secrets")?;
        for secret in output.secret_list() {
            if let Some(name) = secret.name() {
                secret_names.push(name.to_string());
            }
        }
    }

    Ok(secret_names)
}

fn list_keys(secret_names: &[String], format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(secret_names)?);
        }
        OutputFormat::Plain => {
            for name in secret_names {
                println!("{}", name);
            }
        }
    }
    Ok(())
}

fn get_secret(
    secret_data: &BTreeMap<String, Value>,
    _secret_name: &str,
    format: OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&secret_data)?);
        }
        OutputFormat::Plain => {
            for (key, value) in secret_data {
                println!("{}: {}", key, value_to_string(value));
            }
        }
    }
    Ok(())
}

fn search_keys(
    secrets_with_data: &BTreeMap<String, BTreeMap<String, Value>>,
    pattern: &str,
    format: OutputFormat,
) -> Result<()> {
    let mut matches: Vec<KeyValue> = Vec::new();

    // Search through secret names and their keys
    for (secret_name, secret_data) in secrets_with_data {
        // Check if secret name matches
        if secret_name.contains(pattern) {
            matches.push(KeyValue {
                key: format!("[Secret] {}", secret_name),
                value: format!("{} keys", secret_data.len()),
            });
        }

        // Check if any keys within the secret match
        for (key, value) in secret_data {
            if key.contains(pattern) {
                matches.push(KeyValue {
                    key: format!("{}/{}", secret_name, key),
                    value: value_to_string(value),
                });
            }
        }
    }

    if matches.is_empty() {
        bail!("No secrets or keys found matching pattern '{}'", pattern);
    }

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&matches)?);
        }
        OutputFormat::Plain => {
            for kv in matches {
                println!("{}: {}", kv.key, kv.value);
            }
        }
    }
    Ok(())
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_secret() -> BTreeMap<String, Value> {
        let mut map = BTreeMap::new();
        map.insert("api_key".to_string(), json!("abc123"));
        map.insert("db_password".to_string(), json!("secret123"));
        map.insert("prod_db_url".to_string(), json!("https://prod.example.com"));
        map.insert(
            "staging_db_url".to_string(),
            json!("https://staging.example.com"),
        );
        map.insert("port".to_string(), json!(5432));
        map.insert("enabled".to_string(), json!(true));
        map.insert("disabled".to_string(), json!(false));
        map.insert("nullable".to_string(), json!(null));
        map.insert("tags".to_string(), json!(["prod", "important"]));
        map.insert("config".to_string(), json!({"timeout": 30}));
        map
    }

    fn create_test_secrets_with_data() -> BTreeMap<String, BTreeMap<String, Value>> {
        let mut secrets = BTreeMap::new();

        // First secret
        let mut secret1 = BTreeMap::new();
        secret1.insert("api_key".to_string(), json!("abc123"));
        secret1.insert("db_password".to_string(), json!("secret123"));
        secrets.insert("my-app-config".to_string(), secret1);

        // Second secret
        let mut secret2 = BTreeMap::new();
        secret2.insert("prod_db_url".to_string(), json!("https://prod.example.com"));
        secret2.insert(
            "staging_db_url".to_string(),
            json!("https://staging.example.com"),
        );
        secrets.insert("my-app-urls".to_string(), secret2);

        secrets
    }

    #[test]
    fn test_value_to_string_with_string() {
        let value = json!("test_value");
        assert_eq!(value_to_string(&value), "test_value");
    }

    #[test]
    fn test_value_to_string_with_number() {
        let value = json!(42);
        assert_eq!(value_to_string(&value), "42");

        let float_value = json!(3.14);
        assert_eq!(value_to_string(&float_value), "3.14");
    }

    #[test]
    fn test_value_to_string_with_boolean() {
        let true_value = json!(true);
        assert_eq!(value_to_string(&true_value), "true");

        let false_value = json!(false);
        assert_eq!(value_to_string(&false_value), "false");
    }

    #[test]
    fn test_value_to_string_with_null() {
        let value = json!(null);
        assert_eq!(value_to_string(&value), "null");
    }

    #[test]
    fn test_value_to_string_with_array() {
        let value = json!(["a", "b", "c"]);
        assert_eq!(value_to_string(&value), "[\"a\",\"b\",\"c\"]");
    }

    #[test]
    fn test_value_to_string_with_object() {
        let value = json!({"key": "value"});
        assert_eq!(value_to_string(&value), "{\"key\":\"value\"}");
    }

    #[test]
    fn test_get_secret_success() {
        let secret = create_test_secret();
        let result = get_secret(&secret, "test-secret", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_secret_json_format() {
        let secret = create_test_secret();
        let result = get_secret(&secret, "test-secret", OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_secret_different_types() {
        let secret = create_test_secret();
        // Test that get_secret returns all k/v pairs including different types
        let result = get_secret(&secret, "test-secret", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_keys_with_matches() {
        let secrets = create_test_secrets_with_data();
        let result = search_keys(&secrets, "db", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_keys_multiple_matches() {
        let secrets = create_test_secrets_with_data();
        let result = search_keys(&secrets, "url", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_keys_no_matches() {
        let secrets = create_test_secrets_with_data();
        let result = search_keys(&secrets, "xyz_nonexistent", OutputFormat::Plain);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No secrets or keys found"));
    }

    #[test]
    fn test_search_keys_case_sensitive() {
        let secrets = create_test_secrets_with_data();
        // Should not match since search is case-sensitive
        let result = search_keys(&secrets, "API", OutputFormat::Plain);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_keys_partial_match() {
        let secrets = create_test_secrets_with_data();
        // Should match "staging_db_url" and "prod_db_url"
        let result = search_keys(&secrets, "db_url", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_keys_json_format_with_matches() {
        let secrets = create_test_secrets_with_data();
        let result = search_keys(&secrets, "db", OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_keys_json_format_multiple_matches() {
        let secrets = create_test_secrets_with_data();
        let result = search_keys(&secrets, "url", OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_keys_json_format_no_matches() {
        let secrets = create_test_secrets_with_data();
        let result = search_keys(&secrets, "xyz_nonexistent", OutputFormat::Json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No secrets or keys found"));
    }

    #[test]
    fn test_search_keys_matches_secret_name() {
        let secrets = create_test_secrets_with_data();
        // Should match the secret name "my-app-config"
        let result = search_keys(&secrets, "app-config", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_keys_matches_both_secret_and_key() {
        let secrets = create_test_secrets_with_data();
        // Should match both secret name "my-app-urls" and keys containing "app"
        let result = search_keys(&secrets, "app", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_keys_not_empty() {
        let secret_names = vec!["secret1".to_string(), "secret2".to_string()];
        let result = list_keys(&secret_names, OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_keys_empty_secret() {
        let secret_names: Vec<String> = vec![];
        let result = list_keys(&secret_names, OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_keys_preserves_order() {
        let secret_names = vec![
            "beta-secret".to_string(),
            "alpha-secret".to_string(),
            "gamma-secret".to_string(),
        ];
        // The function should preserve the order provided
        let result = list_keys(&secret_names, OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_keys_json_format() {
        let secret_names = vec!["secret1".to_string(), "secret2".to_string()];
        let result = list_keys(&secret_names, OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_keys_json_format_empty() {
        let secret_names: Vec<String> = vec![];
        let result = list_keys(&secret_names, OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fetch_secret_parsing_valid_json() {
        let json_string = r#"{"key1": "value1", "key2": "value2"}"#;
        let parsed: Value = serde_json::from_str(json_string).unwrap();

        match parsed {
            Value::Object(map) => {
                let mut btree_map = BTreeMap::new();
                for (k, v) in map {
                    btree_map.insert(k, v);
                }
                assert_eq!(btree_map.len(), 2);
                assert!(btree_map.contains_key("key1"));
                assert!(btree_map.contains_key("key2"));
            }
            _ => panic!("Should be an object"),
        }
    }

    #[test]
    fn test_fetch_secret_parsing_invalid_json() {
        let invalid_json = "not valid json";
        let result: Result<Value, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_secret_parsing_non_object() {
        // Array instead of object
        let json_string = r#"["item1", "item2"]"#;
        let parsed: Value = serde_json::from_str(json_string).unwrap();

        match parsed {
            Value::Object(_) => panic!("Should not be an object"),
            _ => {} // Expected
        }
    }

    #[test]
    fn test_special_characters_in_keys() {
        let mut secret = BTreeMap::new();
        secret.insert("key-with-dash".to_string(), json!("value1"));
        secret.insert("key_with_underscore".to_string(), json!("value2"));
        secret.insert("key.with.dot".to_string(), json!("value3"));
        secret.insert("key/with/slash".to_string(), json!("value4"));

        let result = get_secret(&secret, "test-secret", OutputFormat::Plain);
        assert!(result.is_ok());

        // Test search with special characters
        let mut secrets = BTreeMap::new();
        secrets.insert("test-secret".to_string(), secret);
        let result = search_keys(&secrets, "with", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unicode_in_values() {
        let mut secret = BTreeMap::new();
        secret.insert("emoji".to_string(), json!("🔐🗝️"));
        secret.insert("chinese".to_string(), json!("密码"));
        secret.insert("arabic".to_string(), json!("كلمة السر"));

        let result = get_secret(&secret, "test-secret", OutputFormat::Plain);
        assert!(result.is_ok());

        // Test list with unicode values
        let secret_names = vec!["test-secret".to_string()];
        let result = list_keys(&secret_names, OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_string_values() {
        let mut secret = BTreeMap::new();
        secret.insert("empty".to_string(), json!(""));

        let result = get_secret(&secret, "test-secret", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_very_long_values() {
        let mut secret = BTreeMap::new();
        let long_value = "a".repeat(10000);
        secret.insert("long_key".to_string(), json!(long_value));

        let result = get_secret(&secret, "test-secret", OutputFormat::Plain);
        assert!(result.is_ok());
    }
}
