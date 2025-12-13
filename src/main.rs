mod cli;
mod config;
mod credinform;
mod fns;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use config::Client;
use credinform::{api, Address};
use serde_json::Value;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    match log::set_logger(&config::CONSOLE_LOGGER) {
        Ok(_) => {
            let level = if args.verbose {
                log::LevelFilter::Info
            } else {
                log::LevelFilter::Warn
            };
            log::set_max_level(level);
        }
        Err(e) => eprintln!("Не удалось настроить логер: {}", e),
    }

    let client = Arc::new(Client::from_toml(args.config.as_str())?);

    if args.full {
        let tax_numbers = cli::collect_tax_numbers(&client)?;
        let cred_ops = tax_numbers.len() * client.credinform_fields().len();
        let tm_ops = if args.trademarks {
            tax_numbers.len()
        } else {
            0
        };
        let fns_ops = tax_numbers.len() * client.fns_fields().len();
        let total = cred_ops + tm_ops + fns_ops;

        if total == 0 {
            eprintln!("Нет работ для выполнения: проверьте config.toml");
            return Ok(());
        }

        let pb = cli::progress_bar(total as u64);
        let token = Arc::new(api::get_token(&client).await?);
        cli::process_all_addresses(&client, &token, &tax_numbers, args.trademarks, Some(&pb))
            .await?;
        cli::process_fns_all(&client, &tax_numbers, Some(&pb)).await?;
        pb.finish_with_message("Готово");
        return Ok(());
    }

    let mut did_work = false;

    if args.cred {
        let tax_number = cli::pick_tax_number(&args, &client)?;
        let token = Arc::new(api::get_token(&client).await?);

        if let Some(field) = args.field.as_ref() {
            let address = Address::new(field);
            let data = cli::process_single_address(&client, &token, &tax_number, &address).await?;
            cli::print_value(&Value::Object(data.data.clone()))?;
        } else {
            cli::process_all_addresses(&client, &token, &[tax_number], args.trademarks, None)
                .await?;
        }
        did_work = true;
    }

    if args.fns {
        let tax_number = cli::pick_tax_number(&args, &client)?;
        if let Some(field) = args.field.as_ref() {
            let response = cli::process_fns_single(&client, &tax_number, field).await?;
            cli::print_value(&response.data)?;
        } else {
            cli::process_fns_all(&client, &[tax_number], None).await?;
        }
        did_work = true;
    }

    if !did_work {
        cli::Args::command().print_help()?;
    }

    Ok(())
}
