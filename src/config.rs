use anyhow::{anyhow, Result};
use chrono::Local;
use log::{Level, Log, Metadata, Record};
use reqwest;
use serde::{Deserialize, Serialize};
use std::fs;
use toml;

pub static CONSOLE_LOGGER: ConsoleLogger = ConsoleLogger;

pub struct ConsoleLogger;

impl Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

#[derive(Deserialize, Serialize)]
pub struct Credinform {
    username: String,
    password: String,
    api_version: String,
    pub tax_numbers: Vec<String>,
    pub fields: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Data {
    pub credinform: Credinform,
}

pub struct Client {
    client: reqwest::Client,
    pub data: Data,
}

impl Client {
    pub fn from_toml(path: &str) -> Result<Client> {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let data = Data {
                    credinform: Credinform {
                        username: "your_username".to_string(),
                        password: "your_password".to_string(),
                        api_version: "1.7".to_string(),
                        tax_numbers: Vec::new(),
                        fields: Vec::new(),
                    },
                };
                let content = toml::to_string(&data)?;
                fs::write(path, content)?;
                return Err(anyhow!("Config file not found. Created new file {}.", path).into());
            }
            Err(e) => return Err(e.into()),
        };

        let data: Data = match toml::de::from_str(&content) {
            Ok(c) => c,
            Err(e) => return Err(anyhow!("Invalid TOML format: {}", e).into()),
        };

        Ok(Client {
            client: reqwest::Client::new(),
            data: data,
        })
    }

    pub fn post<U: reqwest::IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        self.client.post(url)
    }

    pub fn username(&self) -> &str {
        &self.data.credinform.username
    }

    pub fn password(&self) -> &str {
        &self.data.credinform.password
    }

    pub fn api_version(&self) -> &str {
        &self.data.credinform.api_version
    }
}
