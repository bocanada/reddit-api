#[cfg(feature = "stream")]
use sqlx;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("reqwest error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("reddit error: {0}")]
    Reddit(#[from] RedditError),
    #[error("authentication error: {0}")]
    AuthError(#[from] crate::auth::Error),

    #[cfg(feature = "stream")]
    #[error("authentication error: {0}")]
    Sql(#[from] sqlx::Error),
}

#[derive(Debug, thiserror::Error, serde::Deserialize)]
#[serde(untagged)]
pub enum RedditError {
    #[error("{message}: {explanation}")]
    Explained {
        explanation: String,
        message: String,
        reason: String,
    },
    #[error("{message}")]
    Simple { message: String },
    #[error("rate limited")]
    RateLimited,
}
