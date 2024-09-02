use super::{Address, TaxNumber};
use serde::Serialize;
use serde_json::Map;

#[derive(Debug, Clone, Serialize)]
pub struct CredinformData(Map<String, serde_json::Value>);

struct FilePath<'a> {
    address: &'a Address,
    tax_number: &'a TaxNumber,
}

impl<'a> FilePath<'a> {
    fn new(address: &'a Address, tax_number: &'a TaxNumber) -> Self {
        FilePath {
            address,
            tax_number,
        }
    }
}

impl std::fmt::Display for FilePath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}.json", self.tax_number, self.address)
    }
}

impl std::fmt::Display for CredinformData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl CredinformData {
    pub fn new(response: serde_json::Value) -> Result<Self, String> {
        let mut data = Map::new();
        if let Some(data_value) = response.get("data").and_then(|v| v.as_object()) {
            data = data_value.clone();
        }

        Ok(CredinformData(data))
    }

    pub fn save_data(
        &self,
        address: &Address,
        tax_number: &TaxNumber,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = FilePath::new(address, tax_number);
        let mut file = std::fs::File::create(format!("{}", path))?;
        serde_json::to_writer_pretty(&mut file, &self.0)?;
        println!("Data saved to {}", path);
        Ok(())
    }
}
