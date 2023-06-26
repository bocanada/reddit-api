use std::{sync::Arc, time::Duration};

use tokio::time::Instant;

use url::Url;

use crate::auth::{AuthResponse, Authenticator, Error};

#[derive(Clone)]
pub struct Auth {
    client_id: Arc<str>,
    client_secret: Arc<str>,
    username: Arc<str>,
    password: Arc<str>,

    token: Option<Arc<str>>,
    expires_in: Option<Duration>,
    refreshed_at: Option<Instant>,
}

impl Authenticator for Auth {
    fn auth_request(&self, req: reqwest::RequestBuilder) -> super::Result<reqwest::RequestBuilder> {
        let Some(ref token) = self.token else { return Err(Error::LoggedOut) };
        let expires_in = self.expires_in.unwrap();
        let refreshed_at = self.refreshed_at.unwrap();

        if refreshed_at.elapsed() >= expires_in {
            Err(Error::NeedsRefresh)
        } else {
            Ok(req.bearer_auth(token))
        }
    }

    async fn login(&mut self, client: &reqwest::Client) -> super::Result<()> {
        let url = Url::parse("https://www.reddit.com/api/v1/access_token")
            .expect("this to be a valid url");

        let form = [
            ("grant_type", "password"),
            ("username", &self.username),
            ("password", &self.password),
        ];

        let token_response = client
            .post(url)
            .form(&form)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .send()
            .await?
            .error_for_status()?
            .json::<AuthResponse>()
            .await?;

        match token_response {
            AuthResponse::AuthData {
                access_token,
                expires_in,
            } => {
                self.token = Some(Arc::from(access_token));
                self.refreshed_at = Some(Instant::now());
                self.expires_in = Some(Duration::from_secs(expires_in));

                Ok(())
            }
            AuthResponse::ErrorData { error } => Err(Error::Token(error)),
        }
    }

    async fn logout(&mut self, client: &reqwest::Client) -> super::Result<()> {
        match self.token {
            None => Err(Error::LoggedOut),
            Some(ref token) => {
                let form = [
                    ("token", token.as_ref()),
                    ("token_type_hint", "access_token"),
                ];

                client
                    .post("https://www.reddit.com/api/v1/revoke_token")
                    .form(&form)
                    .basic_auth(&self.client_id, Some(&self.client_secret))
                    .send()
                    .await?
                    .error_for_status()?;

                self.token = None;
                self.expires_in = None;
                Ok(())
            }
        }
    }
}

impl Auth {
    pub fn new<S: Into<Arc<str>>>(
        client_id: S,
        client_secret: S,
        username: S,
        password: S,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            username: username.into(),
            password: password.into(),
            token: None,
            expires_in: None,
            refreshed_at: None,
        }
    }
}

impl std::fmt::Debug for Auth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Auth")
            .field("client_id", &self.client_id)
            .field("client_secret", &"[redacted]")
            .field("username", &self.username)
            .field("password", &"[redacted]")
            .field(
                "token",
                if self.token.is_none() {
                    &"not logged in"
                } else {
                    &"[redacted]"
                },
            )
            .finish_non_exhaustive()
    }
}
