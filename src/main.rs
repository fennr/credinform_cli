mod cli;
mod config;
mod credinform;

use anyhow::{anyhow, Result};
use clap::Parser;
use config::Client;
use credinform::api;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arc::new(cli::Args::parse());
    let client = Arc::new(Client::from_toml(args.config.as_str())?);
    let token = Arc::new(api::get_token(&client).await?);

    if args.full {
        cli::process_all_addresses(&client, &token, &args).await?;
    } else if args.trademarks {
        cli::process_trademarks(&client, &token, &args.tax_number).await?;
    } else if let Some(address) = &args.address {
        cli::process_single_address(&client, &token, &args.tax_number, address).await?;
    } else {
        return Err(anyhow!("Invalid arguments, Add '--full' or '--address' to your command"));
    }

    Ok(())
}
