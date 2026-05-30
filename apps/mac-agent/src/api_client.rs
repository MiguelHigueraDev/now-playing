use shared_types::UpdateNowPlayingRequest;
use thiserror::Error;

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    auth_token: String,
}

#[derive(Debug, Error)]
pub enum ApiClientError {
    #[error("http request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API returned status {status}: {body}")]
    UnexpectedStatus { status: u16, body: String },
}

impl ApiClient {
    pub fn new(base_url: String, auth_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_token,
        }
    }

    pub async fn post_now_playing(
        &self,
        payload: &UpdateNowPlayingRequest,
    ) -> Result<(), ApiClientError> {
        let url = format!("{}/api/now-playing", self.base_url);
        let response = self
            .client
            .post(url)
            .bearer_auth(&self.auth_token)
            .json(payload)
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();

        Err(ApiClientError::UnexpectedStatus { status, body })
    }
}
