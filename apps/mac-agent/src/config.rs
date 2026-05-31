use std::time::Duration;

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub api_base_url: String,
    pub auth_token: String,
    pub poll_interval_secs: u64,
}

impl AgentConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let api_base_url = std::env::var("API_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        let auth_token = std::env::var("NOW_PLAYING_TOKEN")
            .context("NOW_PLAYING_TOKEN must be set in the environment")?;

        let poll_interval_secs = Self::parse_poll_interval(
            std::env::var("POLL_INTERVAL_SECS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(3),
        )?;

        Ok(Self {
            api_base_url,
            auth_token,
            poll_interval_secs,
        })
    }

    pub fn default_template() -> Self {
        Self {
            api_base_url: "http://localhost:3000".to_string(),
            auth_token: String::new(),
            poll_interval_secs: 3,
        }
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_secs(self.poll_interval_secs)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.auth_token.trim().is_empty() {
            bail!("auth token must not be empty");
        }

        Self::parse_poll_interval(self.poll_interval_secs)?;
        Ok(())
    }

    fn parse_poll_interval(poll_interval_secs: u64) -> anyhow::Result<u64> {
        if !(2..=5).contains(&poll_interval_secs) {
            bail!("poll_interval_secs must be between 2 and 5");
        }

        Ok(poll_interval_secs)
    }
}
