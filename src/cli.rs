use super::config::Client;
use super::credinform::{api, AccessToken, Address, CredinformData, TaxNumber};
use super::fns::{FnsClient, FnsResponse};
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, warn};
use serde_json::Value;
use std::sync::Arc;
use tokio::task::JoinSet;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[arg(short = 'c', long, help = "Запросить данные из Credinform")]
    pub cred: bool,

    #[arg(short = 'f', long, help = "Запросить данные из FNS")]
    pub fns: bool,

    #[arg(
        long,
        default_value_t = false,
        help = "Выгрузить все ручки описанные в config.toml"
    )]
    pub full: bool,

    #[arg(long, default_value_t = false, help = "Выгрузить товарные знаки")]
    pub trademarks: bool,

    #[arg(
        short = 't',
        long,
        value_name = "INN",
        help = "ИНН компании (для одиночных запросов)"
    )]
    pub tax_number: Option<TaxNumber>,

    #[arg(
        short = 'v',
        long,
        default_value_t = false,
        help = "Подробный вывод (логи)"
    )]
    pub verbose: bool,

    #[arg(
        long,
        value_name = "FIELD",
        conflicts_with = "full",
        help = "Имя field для прямого вывода в stdout"
    )]
    pub field: Option<String>,

    #[arg(long, default_value = "config.toml", help = "Путь к config.toml")]
    pub config: String,

    #[arg(long, value_name = "SHELL", help = "Generate completion script for shell")]
    pub completion: Option<String>,
}

pub async fn process_all_addresses(
    client: &Arc<Client>,
    token: &Arc<AccessToken>,
    tax_numbers: &[TaxNumber],
    trademarks: bool,
    progress: Option<&ProgressBar>,
) -> Result<()> {
    let addresses = Address::from_vec(client.credinform_fields());
    let mut tasks = JoinSet::new();

    for tax_number in tax_numbers {
        let tax_number_clone = tax_number.clone();

        if trademarks {
            let client = Arc::clone(client);
            let token = Arc::clone(token);
            let pb = progress.cloned();
            tasks.spawn(async move {
                if let Err(e) = api::get_trademarks(&client, &token, &tax_number_clone).await {
                    error!("Ошибка выгрузки товарных знаков для {}: {}", tax_number_clone, e);
                }
                if let Some(pb) = pb {
                    pb.inc(1);
                }
                Ok::<(), anyhow::Error>(())
            });
        }

        for address in addresses.clone() {
            let client = Arc::clone(client);
            let token = Arc::clone(token);
            let pb = progress.cloned();
            let tax_number_clone = tax_number.clone();
            tasks.spawn(async move {
                match api::get_data(&client, &token, &tax_number_clone, &address).await {
                    Ok(data) => {
                        if let Err(err) = data.to_file(&address, &tax_number_clone) {
                            error!(
                                "Не удалось сохранить {} для {}: {}",
                                address, tax_number_clone, err
                            );
                        }
                    }
                    Err(err) => {
                        error!(
                            "Не удалось получить {} для {}: {}",
                            address, tax_number_clone, err
                        );
                    }
                }
                if let Some(pb) = pb {
                    pb.inc(1);
                }
                Ok::<(), anyhow::Error>(())
            });
        }
    }

    while let Some(res) = tasks.join_next().await {
        if let Err(e) = res {
            error!("Ошибка выполнения задачи: {}", e);
        }
    }

    Ok(())
}

pub async fn process_single_address(
    client: &Arc<Client>,
    token: &Arc<AccessToken>,
    tax_number: &TaxNumber,
    address: &Address,
) -> Result<CredinformData> {
    let data = api::get_data(client, token, tax_number, address).await?;
    data.to_file(address, tax_number)?;
    Ok(data)
}

pub async fn process_fns_all(
    client: &Arc<Client>,
    tax_numbers: &[TaxNumber],
    progress: Option<&ProgressBar>,
) -> Result<()> {
    let endpoints = client.fns_fields().clone();
    let fns_client = FnsClient::new(client);

    let mut tasks = JoinSet::new();

    for tax_number in tax_numbers {
        for endpoint in endpoints.clone() {
            let fns_client = fns_client.clone();
            let tax_number_clone = tax_number.clone();
            let pb = progress.cloned();
            tasks.spawn(async move {
                match fns_client.fetch(&endpoint, &tax_number_clone).await {
                    Ok(response) => {
                        if let Err(err) = response.to_file(&tax_number_clone) {
                            warn!(
                                "Не удалось сохранить {} для {}: {}",
                                endpoint, tax_number_clone, err
                            );
                        }
                    }
                    Err(err) => {
                        error!("Ошибка FNS {} для {}: {}", endpoint, tax_number_clone, err);
                    }
                }
                if let Some(pb) = pb {
                    pb.inc(1);
                }
                Ok::<(), anyhow::Error>(())
            });
        }
    }

    while let Some(res) = tasks.join_next().await {
        if let Err(e) = res {
            error!("Ошибка выполнения FNS задачи: {}", e);
        }
    }

    Ok(())
}

pub async fn process_fns_single(
    client: &Arc<Client>,
    tax_number: &TaxNumber,
    endpoint: &str,
) -> Result<FnsResponse> {
    let fns_client = FnsClient::new(client);
    let response = fns_client.fetch(endpoint, tax_number).await?;
    response.to_file(tax_number)?;
    Ok(response)
}

pub fn pick_tax_number(args: &Args, client: &Client) -> Result<TaxNumber> {
    if let Some(tn) = args.tax_number.as_ref() {
        return Ok(tn.clone());
    }
    client
        .tax_numbers()
        .first()
        .map(|v| TaxNumber::new(v))
        .ok_or_else(|| anyhow!("Не указан ИНН и пустой список tax_numbers в config"))
}

pub fn collect_tax_numbers(client: &Client) -> Result<Vec<TaxNumber>> {
    let numbers = TaxNumber::from_vec(client.tax_numbers());
    if numbers.is_empty() {
        return Err(anyhow!(
            "В config.toml пустой список [settings].tax_numbers"
        ));
    }
    Ok(numbers)
}

pub fn progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar()),
    );
    pb
}

pub fn print_value(value: &Value) -> Result<()> {
    let json = serde_json::to_string_pretty(value)
        .context("Не удалось сериализовать данные в JSON для stdout")?;
    println!("{}", json);
    Ok(())
}
