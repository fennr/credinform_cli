use crate::config::Client;
use crate::credinform::TaxNumber;
use anyhow::{anyhow, Context, Result};
use log::{debug, info, warn};
use reqwest::Url;
use serde_json::Value;
use std::fs::{create_dir_all, File};
use std::io::Write;

#[derive(Clone)]
pub struct FnsClient {
    http: reqwest::Client,
    base_url: String,
    key: String,
}

impl FnsClient {
    pub fn new(config: &Client) -> Self {
        FnsClient {
            http: reqwest::Client::new(),
            base_url: config.fns_base_url().trim_end_matches('/').to_string(),
            key: config.fns_token().to_string(),
        }
    }

    pub async fn fetch(&self, endpoint: &str, tax_number: &TaxNumber) -> Result<FnsResponse> {
        if self.key.is_empty() {
            return Err(anyhow!(
                "FNS token пуст. Заполните значение в config.toml (fns.token)"
            ));
        }

        let url = build_url(&self.base_url, endpoint)
            .with_context(|| format!("Некорректный путь FNS для {}", endpoint))?;

        debug!("FNS GET {}?req={}", url, tax_number);
        let response = self
            .http
            .get(url.clone())
            .query(&[("req", tax_number.to_string()), ("key", self.key.clone())])
            .send()
            .await
            .with_context(|| format!("Ошибка запроса к {}", url))?
            .error_for_status()
            .with_context(|| format!("FNS вернул ошибку по endpoint {}", endpoint))?;

        let text = response
            .text()
            .await
            .with_context(|| format!("Не удалось получить тело ответа {}", endpoint))?;

        let data = match serde_json::from_str::<Value>(&text) {
            Ok(val) => val,
            Err(e) => {
                warn!(
                    "Не удалось распарсить JSON ответа {}: {}. Сохраняю как текст.",
                    endpoint, e
                );
                Value::String(text)
            }
        };

        Ok(FnsResponse {
            endpoint: endpoint.to_string(),
            data,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FnsResponse {
    pub endpoint: String,
    pub data: Value,
}

impl FnsResponse {
    pub fn to_file(&self, tax_number: &TaxNumber) -> Result<()> {
        let path = format!("fns_data/{}", tax_number);
        create_dir_all(&path).with_context(|| format!("Не удалось создать {}", path))?;
        let file_path = format!("{}/{}.json", path, self.endpoint);
        let mut file = File::create(&file_path)
            .with_context(|| format!("Не удалось создать файл {}", file_path))?;
        serde_json::to_writer_pretty(&mut file, &self.data)
            .with_context(|| format!("Не удалось записать данные в {}", file_path))?;
        writeln!(&mut file)?;
        info!(
            "FNS данные сохранены в {}/{}",
            std::env::current_dir()?.display(),
            file_path
        );
        Ok(())
    }
}

fn build_url(base: &str, endpoint: &str) -> Result<Url> {
    let full = format!(
        "{}/{}",
        base.trim_end_matches('/'),
        endpoint.trim_start_matches('/')
    );
    Url::parse(&full).map_err(|e| anyhow!(e))
}
