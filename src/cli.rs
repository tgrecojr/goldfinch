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

#[derive(Serialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}
