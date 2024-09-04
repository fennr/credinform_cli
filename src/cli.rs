use super::config::Client;
use super::credinform::{api, AccessToken, Address, CredinformData, TaxNumber};
use anyhow::{anyhow, Result};
use clap::Parser;
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

    #[arg(short, long, default_value = "7838368395", help = "ИНН компании")]
    pub tax_number: TaxNumber,

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
    args: &Arc<Args>,
) -> Result<()> {
    let (tx, mut rx) = channel::<Result<CredinformData>>(32);

    for address in Address::from_vec(&client.data.credinform.fields) {
        let client = Arc::clone(client);
        let token = Arc::clone(token);
        let address = Arc::new(address);
        let args = Arc::clone(args);
        let tx = tx.clone();

        tokio::spawn(async move {
            let data = api::get_data(&client, &token, &args.tax_number, &address).await;
            let result = data
                .and_then(|data| data.to_file(&address, &args.tax_number).map(|_| data))
                .map_err(|e| anyhow!("Failed to process address {}: {}", address, e));
            tx.send(result).await.unwrap();
        });
    }

    drop(tx);

    while let Some(result) = rx.recv().await {
        if let Err(e) = result {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}

pub async fn process_single_address(
    client: &Arc<Client>,
    token: &Arc<AccessToken>,
    tax_number: &TaxNumber,
    address: &Address,
) -> Result<()> {
    let data = api::get_data(client, token, tax_number, address).await?;
    data.to_file(address, tax_number)?;
    Ok(())
}
