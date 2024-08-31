use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AccessToken(String);

impl std::str::FromStr for AccessToken {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AccessToken(s.to_string()))
    }
}

impl std::fmt::Display for AccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AccessToken {
    pub fn new(token: &str) -> Self {
        AccessToken(token.to_string())
    }
}
