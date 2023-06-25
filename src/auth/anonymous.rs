use url::Url;

use super::Authenticator;

#[derive(Clone, Debug)]
pub struct Auth;

impl Auth {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Authenticator for Auth {
    #[allow(clippy::pedantic)]
    async fn login(&mut self, _client: &reqwest::Client) -> super::Result<()> {
        Ok(())
    }

    #[allow(clippy::pedantic)]
    async fn logout(&mut self, _client: &reqwest::Client) -> super::Result<()> {
        Ok(())
    }

    fn auth_request(&self, req: reqwest::RequestBuilder) -> super::Result<reqwest::RequestBuilder> {
        Ok(req)
    }

    fn base_url(&self) -> url::Url {
        Url::parse("https://api.reddit.com/").expect("this to be a valid url")
    }
}
