use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

#[derive(Parser)]
#[command(name = "goldfinch")]
#[command(about = "A CLI tool to read key-value pairs from AWS Secrets", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output format
    #[arg(short, long, value_enum, global = true, default_value = "json")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum Commands {
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
pub enum OutputFormat {
    Json,
    Plain,
}

/// One search result.
///
/// `secret` and `key` are kept as separate members rather than joined into a
/// single `secret/key` string: `/` is legal in both AWS secret names and JSON
/// keys, so the joined form is not injective and lets a caller who controls a
/// key name forge attribution to a secret they cannot read.
#[derive(Serialize)]
pub struct KeyValue {
    /// The secret that owns this record.
    pub secret: String,
    /// The matched key within the secret, or `None` when the secret's own name
    /// was what matched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub value: String,
}
