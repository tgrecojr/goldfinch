use anyhow::{bail, Context, Result};
use aws_sdk_secretsmanager::Client;
use serde_json::Value;
use std::collections::BTreeMap;

pub async fn fetch_secret(client: &Client, secret_id: &str) -> Result<BTreeMap<String, Value>> {
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

pub async fn list_all_secrets(client: &Client) -> Result<Vec<String>> {
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

pub async fn fetch_secrets_concurrent(
    client: &Client,
    secret_ids: &[String],
) -> Result<BTreeMap<String, BTreeMap<String, Value>>> {
    let futures: Vec<_> = secret_ids
        .iter()
        .map(|id| async move {
            let data = fetch_secret(client, id).await?;
            Ok::<_, anyhow::Error>((id.clone(), data))
        })
        .collect();

    let results = futures::future::join_all(futures).await;

    let mut secrets_with_data = BTreeMap::new();
    for result in results {
        let (id, data) = result?;
        secrets_with_data.insert(id, data);
    }

    Ok(secrets_with_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
    fn test_fetch_secret_parsing_nested_object() {
        let json_string = r#"{"outer": {"inner": "value"}}"#;
        let parsed: Value = serde_json::from_str(json_string).unwrap();

        match parsed {
            Value::Object(map) => {
                let mut btree_map = BTreeMap::new();
                for (k, v) in map {
                    btree_map.insert(k, v);
                }
                assert_eq!(btree_map.len(), 1);
                assert!(btree_map.contains_key("outer"));
                assert_eq!(btree_map["outer"], json!({"inner": "value"}));
            }
            _ => panic!("Should be an object"),
        }
    }
}
