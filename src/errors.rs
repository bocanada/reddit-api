#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("reqwest error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("reqwest error: {0}")]
    Reddit(#[from] RedditError),
    #[error("authentication error: {0}")]
    AuthError(#[from] crate::auth::Error),
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
}
