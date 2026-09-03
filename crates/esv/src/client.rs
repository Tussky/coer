use coer_model::{Passage, SourceError, VerseSource};
use secrecy::{ExposeSecret, SecretString};

use crate::parse::passage_from;
use crate::wire::EsvResponse;

pub const AUTH_TOKEN_ENV_VAR: &str = "ESV_API_KEY";

const DEFAULT_URL: &str = "https://api.esv.org/v3/passage/text/";

#[derive(Debug, thiserror::Error)]
#[error("{AUTH_TOKEN_ENV_VAR} is not set — put it in .env or the environment")]
pub struct MissingApiKey;

pub struct EsvClient {
    api_key: SecretString,
    url: String,
    http: reqwest::Client,
}

impl EsvClient {
    pub fn new(api_key: SecretString) -> Self {
        Self {
            api_key,
            url: DEFAULT_URL.to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Reads the key from the environment. Loading `.env` first is the view's
    /// job — an adapter should not decide how the process gets configured.
    pub fn from_env() -> Result<Self, MissingApiKey> {
        let key = std::env::var(AUTH_TOKEN_ENV_VAR).map_err(|_| MissingApiKey)?;
        Ok(Self::new(SecretString::from(key)))
    }

    /// Point at a different base URL, for tests against a local stub.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }
}

impl VerseSource for EsvClient {
    /// A plain `async fn`: the trait already declares the returned future
    /// `Send`, and the compiler checks that this body satisfies it.
    async fn fetch(&self, query: &str) -> Result<Passage, SourceError> {
        let response = self
            .http
            .get(&self.url)
            .header(
                "Authorization",
                format!("Token {}", self.api_key.expose_secret()),
            )
            .query(&[("q", query)])
            .send()
            .await
            .map_err(|e| SourceError::Transport(Box::new(e)))?;

        // A dead network and a rejected key are different problems for the
        // user, so they get different variants rather than one panic.
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(SourceError::Unauthorised);
        }
        let response = response
            .error_for_status()
            .map_err(|e| SourceError::Transport(Box::new(e)))?;

        let body: EsvResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Malformed(e.to_string()))?;

        passage_from(body)
    }
}
