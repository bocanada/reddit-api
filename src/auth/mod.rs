mod anonymous;
mod password;
use serde::Deserialize;
use url::Url;

pub type Password = self::password::Auth;
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

pub type Result<T> = std::result::Result<T, Error>;

pub trait Authenticator: Clone + Send + Sync {
    async fn login(&mut self, client: &reqwest::Client) -> Result<()>;

    async fn logout(&mut self, client: &reqwest::Client) -> Result<()>;

    /// # Errors
    /// Returns `Err` if the user isn't logged in.
    /// If the `Authenticator` is [`anonymous::Auth`], then it cannot fail.
    fn auth_request(&self, req: reqwest::RequestBuilder) -> Result<reqwest::RequestBuilder>;

    fn base_url(&self) -> Url {
        Url::parse("https://oauth.reddit.com/").expect("this to be a valid url")
    }
}
