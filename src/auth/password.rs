use std::{
    borrow::Cow,
    time::{SystemTime, UNIX_EPOCH},
};

use url::Url;

use crate::auth::{AuthResponse, Authenticator, Error};

#[derive(Clone)]
pub struct Auth<'a> {
    client_id: Cow<'a, str>,
    client_secret: Cow<'a, str>,
    username: Cow<'a, str>,
    password: Cow<'a, str>,
    token: Option<Cow<'a, str>>,
    expires_in: Option<u128>,
}
impl<'a> Authenticator for Auth<'a> {
    fn auth_request(&self, req: reqwest::RequestBuilder) -> super::Result<reqwest::RequestBuilder> {
        match self.token {
            Some(ref token) => Ok(req.bearer_auth(token)),
            None => Err(Error::LoggedOut),
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
                self.token = Some(Cow::Owned(access_token));
                self.expires_in = Some(
                    u128::from(expires_in * 1000)
                        + SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis(),
                );
                Ok(())
            }
            AuthResponse::ErrorData { error } => Err(Error::TokenError(error)),
        }
    }

    async fn logout(&mut self, client: &reqwest::Client) -> super::Result<()> {
        match self.token {
            None => Err(Error::AlreadyLoggedOut),
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

impl<'a> Auth<'a> {
    pub fn new<S: Into<Cow<'a, str>>>(
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
        }
    }
}

impl std::fmt::Debug for Auth<'_> {
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
