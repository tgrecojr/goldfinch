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
