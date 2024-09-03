use reqwest;
use serde::{Deserialize, Serialize};
use std::fs;
use toml;

#[derive(Deserialize, Serialize)]
pub struct Credinform {
    username: String,
    password: String,
    api_version: String,
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
    pub fn from_toml(path: &str) -> Result<Client, Box<dyn std::error::Error>> {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let data = Data {
                    credinform: Credinform {
                        username: "your_username".to_string(),
                        password: "your_password".to_string(),
                        api_version: "1.7".to_string(),
                        fields: Vec::new(),
                    },
                };
                let content = toml::to_string(&data)?;
                fs::write(path, content)?;
                return Err(format!("Config file not found. Created new file {}.", path).into());
            }
            Err(e) => return Err(e.into()),
        };

        let data: Data = match toml::de::from_str(&content) {
            Ok(c) => c,
            Err(e) => return Err(format!("Invalid TOML format: {}", e).into()),
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
