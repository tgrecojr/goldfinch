use anyhow::{bail, Context, Result};
use aws_sdk_secretsmanager::Client;
use futures::stream::StreamExt;
use serde_json::Value;
use std::collections::BTreeMap;

/// Maximum GetSecretValue calls in flight at once.
///
/// The secret list is enumerated from the account, so its length is influenced
/// by whoever can call CreateSecret. Without a ceiling, one `search` fans out
/// one request per secret, exhausting local sockets and spilling AWS-side
/// throttling onto other workloads.
pub const MAX_CONCURRENT_FETCHES: usize = 8;

/// Maximum number of secrets a single `search` will materialize.
///
/// Bounds resident memory: every fetched body is decrypted and held at once.
/// Set well above any realistic account so normal use is unaffected.
pub const MAX_SECRETS: usize = 10_000;

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
    fetch_all_with(secret_ids, |id| async move { fetch_secret(client, &id).await }).await
}

/// Fetch every id in `secret_ids`, using `fetch` to retrieve one secret.
///
/// Split out from [`fetch_secrets_concurrent`] so the fan-out behaviour can be
/// exercised without an AWS client.
pub async fn fetch_all_with<F, Fut>(
    secret_ids: &[String],
    fetch: F,
) -> Result<BTreeMap<String, BTreeMap<String, Value>>>
where
    F: Fn(String) -> Fut,
    Fut: std::future::Future<Output = Result<BTreeMap<String, Value>>>,
{
    if secret_ids.len() > MAX_SECRETS {
        bail!(
            "too many secrets to fetch in one operation: {} exceeds the limit of {}",
            secret_ids.len(),
            MAX_SECRETS
        );
    }

    let mut pending = futures::stream::iter(secret_ids.iter().map(|id| {
        let id = id.clone();
        let pending = fetch(id.clone());
        async move { Ok::<_, anyhow::Error>((id, pending.await?)) }
    }))
    .buffer_unordered(MAX_CONCURRENT_FETCHES);

    let mut secrets_with_data = BTreeMap::new();
    while let Some(result) = pending.next().await {
        let (id, data) = result?;
        secrets_with_data.insert(id, data);
    }

    Ok(secrets_with_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Records the highest number of fetches in flight at any one moment.
    #[derive(Default)]
    struct ConcurrencyProbe {
        in_flight: AtomicUsize,
        peak: AtomicUsize,
    }

    impl ConcurrencyProbe {
        fn enter(&self) {
            let now = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            self.peak.fetch_max(now, Ordering::SeqCst);
        }

        fn leave(&self) {
            self.in_flight.fetch_sub(1, Ordering::SeqCst);
        }

        fn peak(&self) -> usize {
            self.peak.load(Ordering::SeqCst)
        }
    }

    fn ids(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("secret-{i}")).collect()
    }

    /// VULN-005 (CWE-770): every GetSecretValue call ran concurrently, so peak
    /// concurrency equalled the secret count -- which an attacker influences
    /// via CreateSecret, since the list is enumerated by list_all_secrets.
    #[tokio::test]
    async fn fan_out_is_bounded_regardless_of_secret_count() {
        const N: usize = 500;
        let probe = Arc::new(ConcurrencyProbe::default());

        let result = fetch_all_with(&ids(N), |id| {
            let probe = Arc::clone(&probe);
            async move {
                probe.enter();
                // Yield so other futures can interleave; without a bound they
                // all reach this point together.
                tokio::task::yield_now().await;
                probe.leave();
                let mut data = BTreeMap::new();
                data.insert("k".to_string(), json!(id));
                Ok(data)
            }
        })
        .await
        .expect("all fetches succeed");

        assert_eq!(result.len(), N, "every secret must still be fetched");
        assert!(
            probe.peak() <= MAX_CONCURRENT_FETCHES,
            "fan-out is unbounded: peak concurrency was {} for {N} secrets, \
             expected at most {MAX_CONCURRENT_FETCHES}",
            probe.peak()
        );
    }

    /// VULN-005 second sink region: the result map held every decrypted body
    /// with no ceiling on how many could accumulate.
    #[tokio::test]
    async fn materialization_is_capped() {
        let over = MAX_SECRETS + 1;
        let result = fetch_all_with(&ids(over), |id| async move {
            let mut data = BTreeMap::new();
            data.insert("k".to_string(), json!(id));
            Ok(data)
        })
        .await;

        let err = result.expect_err("a secret count above the cap must be refused");
        assert!(
            err.to_string().contains("too many secrets"),
            "expected a typed over-cap error, got: {err}"
        );
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
