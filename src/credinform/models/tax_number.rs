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
