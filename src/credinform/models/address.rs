use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Address(String);

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

impl std::str::FromStr for Address {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Address(capitalize(s)))
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Address {
    pub fn new(address: &str) -> Self {
        Address(address.to_string())
    }
    pub fn from_vec(addresses: &Vec<String>) -> Vec<Self> {
        addresses
            .iter()
            .map(|a| Address::new(a))
            .collect::<Vec<Self>>()
    }
}
