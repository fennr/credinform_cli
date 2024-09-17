use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TaxNumber(String);

impl std::str::FromStr for TaxNumber {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TaxNumber(s.to_string()))
    }
}

impl std::fmt::Display for TaxNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


impl TaxNumber {
    pub fn new(tax_number: &str) -> Self {
        TaxNumber(tax_number.to_string())
    }

    pub fn from_vec(vec: &Vec<String>) -> Vec<Self> {
        vec.iter().map(|s| TaxNumber::new(s)).collect()
    }
}
