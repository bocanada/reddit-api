mod anonymous;
mod password;
use std::future::Future;

use serde::Deserialize;
use url::Url;

/// Password based [`Authenticator`].
pub type Password = self::password::Auth;
/// Anonymous [`Authenticator`].
pub type Anon = self::anonymous::Auth;

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub(crate) enum AuthResponse {
    AuthData {
        access_token: String,
        expires_in: u64,
    },
    ErrorData {
        error: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("client is logged out.")]
    LoggedOut,
    #[error("token has expired.")]
    NeedsRefresh,
    #[error("reqwest error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("token response error: {0}")]
    Token(String),
}

/// Used in case the [`Authenticator`] calls fail.
pub type Result<T> = std::result::Result<T, Error>;

/// Used to authenticate a [`crate::Client`] instance.
pub trait Authenticator: Clone + Send + Sync {
    /// Logs in to this [`Authenticator`].
    fn login(&mut self, client: &reqwest::Client)
        -> impl Future<Output = Result<()>> + Send + Sync;

    /// Logs out of this [`Authenticator`].
    fn logout(
        &mut self,
        client: &reqwest::Client,
    ) -> impl Future<Output = Result<()>> + Send + Sync;

    /// # Errors
    /// Returns [`Err`] if the user isn't logged in.
    /// If the [`Authenticator`] is [`Anon`], then it cannot fail.
    fn auth_request(&self, req: reqwest::RequestBuilder) -> Result<reqwest::RequestBuilder>;

    /// Returns the base [`Url`] of this Reddit [`Authenticator`].
    fn base_url(&self) -> Url {
        Url::parse("https://oauth.reddit.com/").expect("this to be a valid url")
    }
}
