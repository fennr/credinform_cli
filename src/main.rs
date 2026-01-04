mod cli;
mod completion;
mod config;
mod credinform;
mod fns;
mod services;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use config::Client;
use credinform::Address;
use services::{CredinformService, FnsService, print_value};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    if let Some(shell) = &args.completion {
        return completion::generate_completion_script(shell).map_err(|e| anyhow::anyhow!("Failed to generate completion: {}", e));
    }

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
    let credinform_service = CredinformService::new(Arc::clone(&client));
    let fns_service = FnsService::new(Arc::clone(&client));

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
        let token = Arc::new(credinform_service.get_token().await?);
        credinform_service.process_all_addresses(&token, &tax_numbers, args.trademarks, Some(&pb))
            .await?;
        fns_service.process_all(&tax_numbers, Some(&pb)).await?;
        pb.finish_with_message("Готово");
        return Ok(());
    }

    let mut did_work = false;

    if args.cred {
        let tax_number = cli::pick_tax_number(&args, &client)?;
        let token = Arc::new(credinform_service.get_token().await?);

        if let Some(field) = args.field.as_ref() {
            let address = Address::new(field);
            let data = credinform_service.process_single_address(&token, &tax_number, &address).await?;
            print_value(&serde_json::Value::Object(data.data.clone()))?;
        } else {
            credinform_service.process_all_addresses(&token, &[tax_number], args.trademarks, None)
                .await?;
        }
        did_work = true;
    }

    if args.fns {
        let tax_number = cli::pick_tax_number(&args, &client)?;
        if let Some(field) = args.field.as_ref() {
            let response = fns_service.process_single(&tax_number, field).await?;
            print_value(&response.data)?;
        } else {
            fns_service.process_all(&[tax_number], None).await?;
        }
        did_work = true;
    }

    if !did_work {
        cli::Args::command().print_help()?;
    }

    Ok(())
}
