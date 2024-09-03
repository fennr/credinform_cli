use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SearchCompany {
    pub id: String,
    pub name: String,
}

impl std::fmt::Display for SearchCompany {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SearchCompany {
    pub fn new(id: &str, name: &str) -> Self {
        SearchCompany {
            id: id.to_string(),
            name: name.to_string(),
        }
    }
}
