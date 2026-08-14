mod aws;
mod cli;
mod commands;

use anyhow::Result;
use aws_sdk_secretsmanager::Client;
use clap::Parser;

use aws::{fetch_secret, fetch_secrets_concurrent, list_all_secrets};
use cli::{Cli, Commands};
use commands::{get_secret, list_keys, search_keys};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize AWS config and client
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    match &cli.command {
        Commands::List => {
            let secret_ids = list_all_secrets(&client).await?;
            list_keys(&secret_ids, cli.format)?;
        }
        Commands::Get { secret_name } => {
            // Direct fetch - no list needed (lazy load optimization)
            let secret_data = fetch_secret(&client, secret_name).await?;
            get_secret(&secret_data, cli.format)?;
        }
        Commands::Search { pattern } => {
            let secret_ids = list_all_secrets(&client).await?;
            let outcome = fetch_secrets_concurrent(&client, &secret_ids).await?;
            // Report unreadable secrets on stderr so they are visible without
            // suppressing results for the ones that were readable.
            for (id, err) in &outcome.failures {
                eprintln!("warning: skipping secret '{}': {}", id, err);
            }
            search_keys(&outcome.secrets, pattern, cli.format)?;
        }
    }

    Ok(())
}
