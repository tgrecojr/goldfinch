use anyhow::{Context, Result, bail};
use aws_sdk_secretsmanager::Client;
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;

#[derive(Parser)]
#[command(name = "goldfinch")]
#[command(about = "A CLI tool to read key-value pairs from an AWS Secret", long_about = None)]
struct Cli {
    /// AWS Secret name or ARN (can also be set via GOLDFINCH_SECRET env var)
    #[arg(short, long)]
    secret: Option<String>,

    #[command(subcommand)]
    command: Commands,

    /// Output format
    #[arg(short, long, value_enum, global = true, default_value = "json")]
    format: OutputFormat,
}

#[derive(Subcommand)]
enum Commands {
    /// List all keys in the secret
    List,

    /// Get a specific key's value by exact name
    Get {
        /// The exact key name
        key: String,
    },

    /// Search for keys matching a pattern
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

    let secret_id = cli.secret
        .or_else(|| env::var("GOLDFINCH_SECRET").ok())
        .context(
            "Secret name is required. Provide it via --secret flag or GOLDFINCH_SECRET environment variable"
        )?;

    // Initialize AWS config and client
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    // Fetch the secret value
    let secret_data = fetch_secret(&client, &secret_id).await?;

    match cli.command {
        Commands::List => list_keys(&secret_data, cli.format)?,
        Commands::Get { key } => get_key(&secret_data, &key, cli.format)?,
        Commands::Search { pattern } => search_keys(&secret_data, &pattern, cli.format)?,
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

    let json: Value = serde_json::from_str(secret_string)
        .context("Secret value is not valid JSON")?;

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

fn list_keys(secret_data: &BTreeMap<String, Value>, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let keys: Vec<&String> = secret_data.keys().collect();
            println!("{}", serde_json::to_string_pretty(&keys)?);
        }
        OutputFormat::Plain => {
            for key in secret_data.keys() {
                println!("{}", key);
            }
        }
    }
    Ok(())
}

fn get_key(secret_data: &BTreeMap<String, Value>, key: &str, format: OutputFormat) -> Result<()> {
    let value = secret_data
        .get(key)
        .context(format!("Key '{}' not found in secret", key))?;

    match format {
        OutputFormat::Json => {
            let kv = KeyValue {
                key: key.to_string(),
                value: value_to_string(value),
            };
            println!("{}", serde_json::to_string_pretty(&kv)?);
        }
        OutputFormat::Plain => {
            println!("{}", value_to_string(value));
        }
    }
    Ok(())
}

fn search_keys(
    secret_data: &BTreeMap<String, Value>,
    pattern: &str,
    format: OutputFormat,
) -> Result<()> {
    let matches: Vec<KeyValue> = secret_data
        .iter()
        .filter(|(key, _)| key.contains(pattern))
        .map(|(key, value)| KeyValue {
            key: key.clone(),
            value: value_to_string(value),
        })
        .collect();

    if matches.is_empty() {
        bail!("No keys found matching pattern '{}'", pattern);
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
        map.insert("staging_db_url".to_string(), json!("https://staging.example.com"));
        map.insert("port".to_string(), json!(5432));
        map.insert("enabled".to_string(), json!(true));
        map.insert("disabled".to_string(), json!(false));
        map.insert("nullable".to_string(), json!(null));
        map.insert("tags".to_string(), json!(["prod", "important"]));
        map.insert("config".to_string(), json!({"timeout": 30}));
        map
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
    fn test_get_key_success() {
        let secret = create_test_secret();
        let result = get_key(&secret, "api_key", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_key_not_found() {
        let secret = create_test_secret();
        let result = get_key(&secret, "nonexistent_key", OutputFormat::Plain);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_get_key_different_types() {
        let secret = create_test_secret();

        // String value
        let result = get_key(&secret, "api_key", OutputFormat::Plain);
        assert!(result.is_ok());

        // Number value
        let result = get_key(&secret, "port", OutputFormat::Plain);
        assert!(result.is_ok());

        // Boolean value
        let result = get_key(&secret, "enabled", OutputFormat::Plain);
        assert!(result.is_ok());

        // Null value
        let result = get_key(&secret, "nullable", OutputFormat::Plain);
        assert!(result.is_ok());

        // Array value
        let result = get_key(&secret, "tags", OutputFormat::Plain);
        assert!(result.is_ok());

        // Object value
        let result = get_key(&secret, "config", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_keys_with_matches() {
        let secret = create_test_secret();
        let result = search_keys(&secret, "db", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_keys_multiple_matches() {
        let secret = create_test_secret();
        let result = search_keys(&secret, "url", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_keys_no_matches() {
        let secret = create_test_secret();
        let result = search_keys(&secret, "xyz_nonexistent", OutputFormat::Plain);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No keys found"));
    }

    #[test]
    fn test_search_keys_case_sensitive() {
        let secret = create_test_secret();
        // Should not match since search is case-sensitive
        let result = search_keys(&secret, "API", OutputFormat::Plain);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_keys_partial_match() {
        let secret = create_test_secret();
        // Should match "staging_db_url" and "prod_db_url"
        let result = search_keys(&secret, "db_url", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_keys_not_empty() {
        let secret = create_test_secret();
        let result = list_keys(&secret, OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_keys_empty_secret() {
        let secret = BTreeMap::new();
        let result = list_keys(&secret, OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_keys_preserves_order() {
        let secret = create_test_secret();
        // BTreeMap should maintain alphabetical order
        let keys: Vec<&String> = secret.keys().collect();
        let mut sorted_keys = keys.clone();
        sorted_keys.sort();
        assert_eq!(keys, sorted_keys);
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

        let result = get_key(&secret, "key-with-dash", OutputFormat::Plain);
        assert!(result.is_ok());

        let result = search_keys(&secret, "with", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unicode_in_values() {
        let mut secret = BTreeMap::new();
        secret.insert("emoji".to_string(), json!("🔐🗝️"));
        secret.insert("chinese".to_string(), json!("密码"));
        secret.insert("arabic".to_string(), json!("كلمة السر"));

        let result = get_key(&secret, "emoji", OutputFormat::Plain);
        assert!(result.is_ok());

        let result = list_keys(&secret, OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_string_values() {
        let mut secret = BTreeMap::new();
        secret.insert("empty".to_string(), json!(""));

        let result = get_key(&secret, "empty", OutputFormat::Plain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_very_long_values() {
        let mut secret = BTreeMap::new();
        let long_value = "a".repeat(10000);
        secret.insert("long_key".to_string(), json!(long_value));

        let result = get_key(&secret, "long_key", OutputFormat::Plain);
        assert!(result.is_ok());
    }
}
