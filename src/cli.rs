use super::config::Client;
use super::credinform::{api, AccessToken, Address, CredinformData, TaxNumber};
use super::fns::FnsClient;
use anyhow::{anyhow, Result};
use clap::Parser;
use log::error;
use std::sync::Arc;
use tokio::sync::mpsc::channel;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[arg(short, long, conflicts_with = "full", help = "Имя ручки в credinform")]
    pub address: Option<Address>,

    #[arg(
        short,
        long,
        conflicts_with = "address",
        default_value = "false",
        help = "Выгрузить все ручки описанные в config.toml"
    )]
    pub full: bool,

    #[arg(long, default_value = "false", help = "Выгрузить товарные знаки")]
    pub trademarks: bool,

    #[arg(short, long, default_value = "7838368395", help = "ИНН компании")]
    pub tax_number: TaxNumber,

    #[arg(long, default_value = "false", help = "Выгрузить данные из API FNS")]
    pub fns: bool,

    #[arg(
        short,
        long,
        default_value = "config.toml",
        help = "Путь к config.toml"
    )]
    pub config: String,
}

pub async fn process_all_addresses(
    client: &Arc<Client>,
    token: &Arc<AccessToken>,
    trademarks: bool,
) -> Result<()> {
    let (tx, mut rx) = channel::<Result<CredinformData>>(32);

    for tax_number in TaxNumber::from_vec(&client.data.credinform.tax_numbers) {
        let tax_number = Arc::new(tax_number);

        if trademarks {
            let client = Arc::clone(client);
            let token = Arc::clone(token);
            let tax_number = Arc::clone(&tax_number);
            let tx = tx.clone();
            tokio::spawn(async move {
                let result = api::get_trademarks(&client, &token, &tax_number).await;
                tx.send(result).await.unwrap();
            });
        }

        for address in Address::from_vec(&client.data.credinform.fields) {
            let client = Arc::clone(client);
            let token = Arc::clone(token);
            let tax_number = Arc::clone(&tax_number);
            let address = Arc::new(address);
            let tx = tx.clone();

            tokio::spawn(async move {
                let data = api::get_data(&client, &token, &tax_number, &address).await;
                let result = data
                    .and_then(|data| data.to_file(&address, &tax_number).map(|_| data))
                    .map_err(|e| anyhow!("Failed to process address {}: {}", address, e));
                tx.send(result).await.unwrap();
            });
        }
    }

    drop(tx);

    while let Some(result) = rx.recv().await {
        if let Err(e) = result {
            error!("Error: {}", e);
        }
    }

    Ok(())
}

pub async fn process_single_address(
    client: &Arc<Client>,
    token: &Arc<AccessToken>,
    tax_number: &Arc<TaxNumber>,
    address: &Address,
) -> Result<()> {
    let data = api::get_data(client, token, tax_number, address).await?;
    data.to_file(address, tax_number)?;
    Ok(())
}

pub async fn process_trademarks(
    client: &Arc<Client>,
    token: &Arc<AccessToken>,
    tax_number: &TaxNumber,
) -> Result<()> {
    api::get_trademarks(client, token, tax_number).await?;
    Ok(())
}

pub async fn process_fns_all(client: &Arc<Client>) -> Result<()> {
    for tax_number in TaxNumber::from_vec(&client.data.credinform.tax_numbers) {
        process_fns_single(client, &tax_number).await?;
    }

    Ok(())
}

pub async fn process_fns_single(client: &Arc<Client>, tax_number: &TaxNumber) -> Result<()> {
    let fns_client = FnsClient::new(client);
    for endpoint in client.fns_fields() {
        let response = fns_client.fetch(endpoint, tax_number).await?;
        response.to_file(tax_number)?;
    }
    Ok(())
}
