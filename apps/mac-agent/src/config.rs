use std::time::Duration;

use anyhow::{bail, Context};

pub struct AgentConfig {
    pub api_base_url: String,
    pub auth_token: String,
    pub poll_interval: Duration,
}

impl AgentConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let api_base_url = std::env::var("API_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        let auth_token = std::env::var("NOW_PLAYING_TOKEN")
            .context("NOW_PLAYING_TOKEN must be set in the environment")?;

        let poll_interval_secs: u64 = std::env::var("POLL_INTERVAL_SECS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(3);

        if !(2..=5).contains(&poll_interval_secs) {
            bail!("POLL_INTERVAL_SECS must be between 2 and 5");
        }

        Ok(Self {
            api_base_url,
            auth_token,
            poll_interval: Duration::from_secs(poll_interval_secs),
        })
    }
}
