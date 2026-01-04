use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::AccessToken;

const CACHE_FILE: &str = ".credinform_token_cache.json";
const TOKEN_LIFETIME_HOURS: i64 = 23;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedToken {
    pub token: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

impl CachedToken {
    pub fn new(token: String, username: String) -> Self {
        Self {
            token,
            username,
            created_at: Utc::now(),
        }
    }

    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.created_at);
        age.num_hours() < TOKEN_LIFETIME_HOURS
    }

    pub fn to_access_token(&self) -> AccessToken {
        AccessToken::new(&self.token)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCache {
    tokens: Vec<CachedToken>,
}

impl TokenCache {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    pub fn load() -> Result<Self> {
        if !Path::new(CACHE_FILE).exists() {
            return Ok(Self::new());
        }

        let content =
            fs::read_to_string(CACHE_FILE).context("Не удалось прочитать файл кэша токенов")?;

        let cache: TokenCache =
            serde_json::from_str(&content).context("Не удалось десериализовать кэш токенов")?;

        Ok(cache)
    }

    pub fn save(&self) -> Result<()> {
        let content =
            serde_json::to_string_pretty(&self).context("Не удалось сериализовать кэш токенов")?;

        fs::write(CACHE_FILE, content).context("Не удалось сохранить кэш токенов")?;

        Ok(())
    }

    pub fn get_token(&self, username: &str) -> Option<AccessToken> {
        self.tokens
            .iter()
            .find(|cached| cached.username == username && cached.is_valid())
            .map(|cached| cached.to_access_token())
    }

    pub fn add_token(&mut self, token: String, username: String) -> AccessToken {
        self.tokens.retain(|cached| cached.username != username);

        let cached = CachedToken::new(token, username);
        let access_token = cached.to_access_token();
        self.tokens.push(cached);

        access_token
    }

    pub fn clear_invalid_tokens(&mut self) {
        self.tokens.retain(|cached| cached.is_valid());
    }

    pub fn clear_for_user(&mut self, username: &str) {
        self.tokens.retain(|cached| cached.username != username);
    }
}

impl Default for TokenCache {
    fn default() -> Self {
        Self::new()
    }
}
