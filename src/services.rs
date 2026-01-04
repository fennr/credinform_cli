use crate::config::Client;
use crate::credinform::{api, Address, CredinformData, TaxNumber, AccessToken};
use crate::fns::{FnsClient, FnsResponse};
use anyhow::{anyhow, Result};
use indicatif::ProgressBar;
use serde_json::Value;
use std::sync::Arc;
use tokio::task::JoinSet;
use log::{error, warn};

pub struct CredinformService {
    client: Arc<Client>,
    http_client: Arc<reqwest::Client>,
}

impl CredinformService {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            http_client: Arc::new(reqwest::Client::new()),
        }
    }

    pub async fn process_all_addresses(
        &self,
        token: &Arc<AccessToken>,
        tax_numbers: &[TaxNumber],
        trademarks: bool,
        progress: Option<&ProgressBar>,
    ) -> Result<()> {
        let addresses = Address::from_vec(self.client.credinform_fields());
        let mut tasks = JoinSet::new();

        for tax_number in tax_numbers {
            let tax_number_clone = tax_number.clone();
            let client_for_trademarks = Arc::clone(&self.client);
            let http_client_for_trademarks = Arc::clone(&self.http_client);
            let token_for_trademarks = Arc::clone(token);
            let pb_for_trademarks = progress.cloned();

            if trademarks {
                tasks.spawn(async move {
                    if let Err(e) = Self::get_trademarks_async(
                        &client_for_trademarks, &http_client_for_trademarks, &token_for_trademarks, &tax_number_clone
                    ).await {
                        error!("Ошибка выгрузки товарных знаков для {}: {}", tax_number_clone, e);
                    }
                    if let Some(pb) = pb_for_trademarks {
                        pb.inc(1);
                    }
                    Ok::<(), anyhow::Error>(())
                });
            }

            for address in addresses.clone() {
                let client_for_data = Arc::clone(&self.client);
                let http_client_for_data = Arc::clone(&self.http_client);
                let token_for_data = Arc::clone(token);
                let address = address.clone();
                let tax_number_clone = tax_number.clone();
                let pb_for_data = progress.cloned();

                tasks.spawn(async move {
                    match Self::get_data_async(
                        &client_for_data, &http_client_for_data, &token_for_data, &tax_number_clone, &address
                    ).await {
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
                    if let Some(pb) = pb_for_data {
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
        &self,
        token: &Arc<AccessToken>,
        tax_number: &TaxNumber,
        address: &Address,
    ) -> Result<CredinformData> {
        let data = Self::get_data_async(
            &self.client,
            &self.http_client,
            token,
            tax_number,
            address,
        ).await?;
        data.to_file(address, tax_number)?;
        Ok(data)
    }

    async fn get_data_async(
        client: &Arc<Client>,
        http_client: &Arc<reqwest::Client>,
        token: &Arc<AccessToken>,
        tax_number: &TaxNumber,
        address: &Address,
    ) -> Result<CredinformData> {
        api::get_data_with_client(client, http_client, token, tax_number, address).await
    }

    async fn get_trademarks_async(
        client: &Arc<Client>,
        http_client: &Arc<reqwest::Client>,
        token: &Arc<AccessToken>,
        tax_number: &TaxNumber,
    ) -> Result<CredinformData> {
        api::get_trademarks_with_client(client, http_client, token, tax_number).await
    }

    pub async fn get_token(&self) -> Result<AccessToken> {
        api::get_token_with_client(&self.client, &self.http_client).await
    }
}

pub struct FnsService {
    client: Arc<Client>,
    http_client: Arc<reqwest::Client>,
}

impl FnsService {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            http_client: Arc::new(reqwest::Client::new()),
        }
    }

    pub async fn process_all(
        &self,
        tax_numbers: &[TaxNumber],
        progress: Option<&ProgressBar>,
    ) -> Result<()> {
        let endpoints = self.client.fns_fields().clone();
        let fns_client = FnsClient::new_with_client(&self.client, &self.http_client);

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

    pub async fn process_single(
        &self,
        tax_number: &TaxNumber,
        endpoint: &str,
    ) -> Result<FnsResponse> {
        let fns_client = FnsClient::new_with_client(&self.client, &self.http_client);
        let response = fns_client.fetch(endpoint, tax_number).await?;
        response.to_file(tax_number)?;
        Ok(response)
    }
}

pub fn print_value(value: &Value) -> Result<()> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| anyhow!("Не удалось сериализовать данные в JSON для stdout: {}", e))?;
    println!("{}", json);
    Ok(())
}