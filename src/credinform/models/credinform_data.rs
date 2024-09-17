use super::{Address, TaxNumber};
use anyhow::Result;
use serde::Serialize;
use serde_json::Map;
use log::info;

#[derive(Debug, Clone, Serialize)]
pub struct CredinformData {
    pub company_name: String,
    pub data: Map<String, serde_json::Value>,
}

impl std::fmt::Display for CredinformData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

impl CredinformData {
    pub fn new(name: &str, response: serde_json::Value) -> Result<Self, String> {
        let mut data = Map::new();
        if let Some(data_value) = response.get("data").and_then(|v| v.as_object()) {
            data = data_value.clone();
        }

        Ok(CredinformData {
            company_name: name.to_string(),
            data,
        })
    }

    pub fn to_file(&self, address: &Address, tax_number: &TaxNumber) -> Result<()> {
        let path = format!("credinform_data/{} - {}", tax_number, &self.company_name);
        std::fs::create_dir_all(&path)?;
        let file_path = format!("{}/{}.json", path, address);
        let mut file = std::fs::File::create(&file_path)?;
        serde_json::to_writer_pretty(&mut file, &self.data)?;
        info!(
            "Data saved to {}/{}",
            std::env::current_dir()?.display(),
            file_path
        );
        Ok(())
    }
}
