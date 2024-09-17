mod cli;
mod config;
mod credinform;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use config::Client;
use credinform::api;
use log;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    match log::set_logger(&config::CONSOLE_LOGGER) {
        Ok(_) => log::set_max_level(log::LevelFilter::Info),
        Err(e) => {
            // handle the error here
            eprintln!("Error setting logger: {}", e);
        }
    }

    let args = cli::Args::parse();
    let client = Arc::new(Client::from_toml(args.config.as_str())?);
    let token = Arc::new(api::get_token(&client).await?);
    let tax_number = Arc::new(args.tax_number.clone());


    match (args.full, args.address.is_some(), args.trademarks) {
        (true, _, _) => cli::process_all_addresses(&client, &token, args.trademarks).await?,
        (false, true, true) => {
            cli::process_single_address(&client, &token, &tax_number, &args.address.unwrap()).await?;
            cli::process_trademarks(&client, &token, &tax_number).await?
        },
        (false, true, false) => cli::process_single_address(&client, &token, &tax_number, &args.address.unwrap()).await?,
        (false, false, true) => cli::process_trademarks(&client, &token, &tax_number).await?,
        _ => cli::Args::command().print_help()?
    }
    
    Ok(())
}
