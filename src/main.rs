mod cli;
mod config;
mod credinform;
mod fns;

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
    let tax_number = Arc::new(args.tax_number.clone());

    let need_credinform = args.full || args.address.is_some() || args.trademarks;
    let token = if need_credinform {
        Some(Arc::new(api::get_token(&client).await?))
    } else {
        None
    };

    if args.full {
        if let Some(token) = token.as_ref() {
            cli::process_all_addresses(&client, token, args.trademarks).await?;
        }
        cli::process_fns_all(&client).await?;
        return Ok(());
    }

    match (args.address.is_some(), args.trademarks, args.fns) {
        (true, true, _) => {
            if let Some(token) = token.as_ref() {
                cli::process_single_address(&client, token, &tax_number, &args.address.unwrap())
                    .await?;
                cli::process_trademarks(&client, token, &tax_number).await?;
            }
            if args.fns {
                cli::process_fns_single(&client, &tax_number).await?;
            }
        }
        (true, false, _) => {
            if let Some(token) = token.as_ref() {
                cli::process_single_address(&client, token, &tax_number, &args.address.unwrap())
                    .await?;
            }
            if args.fns {
                cli::process_fns_single(&client, &tax_number).await?;
            }
        }
        (false, true, false) => {
            if let Some(token) = token.as_ref() {
                cli::process_trademarks(&client, token, &tax_number).await?;
            }
        }
        (false, false, true) => {
            cli::process_fns_single(&client, &tax_number).await?;
        }
        _ => cli::Args::command().print_help()?,
    };

    Ok(())
}
