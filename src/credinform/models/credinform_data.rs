use super::Address;
use serde::Serialize;
use serde_json::Map;

#[derive(Debug, Clone, Serialize)]
pub struct CredinformData(Map<String, serde_json::Value>);

impl std::fmt::Display for CredinformData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl CredinformData {
    pub fn new(response: serde_json::Value) -> Self {
        let data = response
            .get("data")
            .ok_or("Failed to get data")
            .unwrap()
            .as_object()
            .ok_or("Failed to get data")
            .unwrap();

        CredinformData(data.clone())
    }
    pub fn save_data(&self, address: &Address) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = std::fs::File::create(format!("{}.json", address))?;
        serde_json::to_writer_pretty(&mut file, &self.0)?;
        println!("Data saved to {}.json", address);
        Ok(())
    }
}
