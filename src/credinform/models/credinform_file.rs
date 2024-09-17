use super::TaxNumber;
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use log::info;
use serde::Serialize;
use std::fs;
use std::io::Write;

#[derive(Debug, Clone, Serialize)]
pub struct CredinformFile {
    company: String,
    name: String,
    ext: String,
    bytes: String,
}

impl std::fmt::Display for CredinformFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.name, self.ext)
    }
}

impl CredinformFile {
    pub fn new(company_name: &str, file_data: &serde_json::Value) -> Result<Self, String> {
        let name = file_data
            .get("fileName")
            .and_then(|v| v.as_str())
            .ok_or("Failed to get fileName")?
            .to_string();

        let ext = file_data
            .get("extension")
            .and_then(|v| v.as_str())
            .ok_or("Failed to get extension")?
            .to_string();

        let bytes = file_data
            .get("fileData")
            .and_then(|v| v.as_str())
            .ok_or("Failed to get fileData")?
            .to_string();

        Ok(CredinformFile {
            company: company_name.to_string(),
            name,
            ext,
            bytes,
        })
    }

    pub fn save(&self, tax_number: &TaxNumber) -> Result<()> {
        let path = format!("credinform_data/{} - {}/files", tax_number, &self.company);
        fs::create_dir_all(&path)?;
        let file_path = format!("{}/{}.{}", path, self.name, self.ext.to_ascii_lowercase());
        let decoded_data = general_purpose::STANDARD.decode(&self.bytes)?;
        let mut file = fs::File::create(&file_path)?;
        file.write_all(&decoded_data)?;
        info!(
            "File saved to {}/{}",
            std::env::current_dir()?.display(),
            file_path
        );
        Ok(())
    }
}
