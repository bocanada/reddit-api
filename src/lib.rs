#![deny(clippy::all, clippy::pedantic)]
#![feature(async_iterator)]
#![feature(async_fn_in_trait)]
pub mod auth;
pub mod errors;
pub mod multireddit;
pub(crate) mod response;
pub mod subreddit;

use std::path::{Path, PathBuf};
#[cfg(feature = "shared_auth")]
use std::sync::Arc;

use auth::Authenticator;
use multireddit::{response::MultiResponse, MultiPath, Multireddit};
use response::Generic;
use serde::de::DeserializeOwned;
use subreddit::Subreddit;
use tracing::trace;
use url::Url;

pub type Result<T> = std::result::Result<T, crate::errors::Error>;

#[derive(Clone)]
pub struct Client<A: Authenticator> {
    #[cfg(feature = "shared_auth")]
    authenticator: Arc<tokio::sync::RwLock<A>>,
    #[cfg(not(feature = "shared_auth"))]
    authenticator: A,
    client: reqwest::Client,
    base_url: Url,
}

pub trait Stream {
    type Item;

    fn stop(&mut self);

    async fn next(&mut self) -> Option<Self::Item>;
}

impl<A: Authenticator + Send + Sync> Client<A> {
    #[must_use]
    pub fn subreddit(&self, subreddit: &str) -> Subreddit<A> {
        Subreddit::new(subreddit, self.clone())
    }

    /// # Errors
    /// This function may error if `Reddit` returns an error.
    /// It may also return [`Err`] if the underlying [`reqwest::Client::get`] call fails.
    /// Or if the underlying [`reqwest::Response::json`] fails.
    pub async fn multi(&self, multipath: MultiPath) -> Result<Multireddit<A>> {
        let path: PathBuf = multipath.into();

        match self.get_json::<MultiResponse>(&path, &[]).await? {
            Generic::LabeledMulti { data } => Ok(data.into_usable(self)),
            other => unimplemented!("expected LabeledMulti but got {}", other.kind_name()),
        }
    }

    pub fn new(authenticator: A, user_agent: &str) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .build()
            .expect("this to be a valid client");

        Self {
            base_url: authenticator.base_url(),

            #[cfg(not(feature = "shared_auth"))]
            authenticator,
            #[cfg(feature = "shared_auth")]
            authenticator: Arc::new(tokio::sync::RwLock::new(authenticator)),

            client,
        }
    }

    /// # Errors
    /// Returns `Err` if the underlying [`reqwest::Client::post`] call fails.
    #[tracing::instrument(name = "Logging in", skip_all)]
    pub async fn login(&mut self) -> Result<()> {
        #[cfg(feature = "shared_auth")]
        {
            let mut guard = self.authenticator.write().await;
            guard.login(&self.client).await?;
        }
        #[cfg(not(feature = "shared_auth"))]
        {
            self.authenticator.login(&self.client).await?;
        }
        Ok(())
    }

    /// # Errors
    /// Returns `Err` if the underlying [`reqwest::Client::post`] call fails.
    #[tracing::instrument(name = "Logging out", skip_all)]
    pub async fn logout(&mut self) -> Result<()> {
        #[cfg(feature = "shared_auth")]
        {
            let mut guard = self.authenticator.write().await;
            guard.logout(&self.client).await?;
        }
        #[cfg(not(feature = "shared_auth"))]
        {
            self.authenticator.logout(&self.client).await?;
        }
        Ok(())
    }

    #[tracing::instrument(name = "GET", skip_all, fields(path = %path.display()))]
    pub(crate) async fn get_json<T: DeserializeOwned>(
        &self,
        path: &Path,
        params: &[(&str, String)],
    ) -> Result<T> {
        let url = build_url(self.base_url.clone(), path, params);

        trace!(url = %url, "fetching");

        let mut req = self.client.get(url);

        #[cfg(feature = "shared_auth")]
        {
            let guard = self.authenticator.read().await;
            req = guard.auth_request(req)?;
        }

        #[cfg(not(feature = "shared_auth"))]
        {
            req = self.authenticator.auth_request(req)?;
        }

        let resp = req.send().await?;
        if resp.status().is_client_error() || resp.status().is_server_error() {
            Err(crate::errors::Error::Reddit(resp.json().await?))
        } else {
            Ok(resp.json().await?)
        }
    }
}

impl Client<crate::auth::anonymous::Auth> {
    #[must_use]
    pub fn anonymous(user_agent: &str) -> Self {
        Self::new(crate::auth::anonymous::Auth::new(), user_agent)
    }
}

pub(crate) fn build_url(mut base: Url, path: &Path, params: &[(&str, String)]) -> Url {
    // Build the path
    {
        let mut segments = base
            .path_segments_mut()
            .expect("expected the url to be a base");

        segments.extend(path.iter().filter_map(std::ffi::OsStr::to_str));
    };

    // Build the params
    {
        let mut query_params = base.query_pairs_mut();
        query_params.extend_pairs(params);
        query_params.append_pair("raw_json", "1");
    };

    base.clone()
}

impl<A> std::fmt::Debug for Client<A>
where
    A: Authenticator,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::auth::{anonymous, password, Authenticator};
    use crate::subreddit::feed;
    use crate::{build_url, Client};
    use dotenv::{dotenv, var};

    #[tokio::test]
    async fn test_anon_auth() {
        dotenv().unwrap();
        let username = var("REDDIT_USERNAME").unwrap();
        let pkg_name = var("CARGO_PKG_NAME").unwrap();

        let mut client = Client::anonymous(&format!("{pkg_name} (by u/{username})"));

        assert!(client.login().await.is_ok());

        let sub = client.subreddit("argentina");
        let latest = sub.latest().await;
        assert!(latest.is_ok());
    }

    #[tokio::test]
    async fn test_password_auth() {
        dotenv().unwrap();

        let username = var("REDDIT_USERNAME").unwrap();
        let pkg_name = var("CARGO_PKG_NAME").unwrap();
        let user_agent = format!("{pkg_name} (by u/{username})");

        let password = var("REDDIT_PASSWORD").unwrap();
        let client_id = var("REDDIT_CLIENT_ID").unwrap();
        let client_secret = var("REDDIT_CLIENT_SECRET").unwrap();

        let auth = password::Auth::new(client_id, client_secret, username, password);

        let mut client = Client::new(auth, &user_agent);
        assert!(client.login().await.is_ok());

        let sub = client.subreddit("argentina");
        let about = sub.about().await;
        assert!(about.is_ok());
        let latest = sub.latest().await;
        assert!(latest.is_ok());

        let latest = sub
            .feed_with_options(
                feed::Sort::Controversial(feed::TimePeriod::ThisYear),
                feed::Options::default(),
            )
            .await;

        assert!(latest.is_ok());
    }

    #[test]
    fn test_build_url() {
        let auth = password::Auth::new("id", "secret", "user", "password");
        let base = auth.base_url();

        let path: PathBuf = ["r", "argentina", "controversial", ".json"]
            .iter()
            .collect();

        let mut params: Vec<(&str, String)> = feed::Options::default().into();

        params.push(("t", feed::TimePeriod::ThisYear.as_str().to_string()));

        assert_eq!(
            build_url(base, &path, &params).to_string(),
            "https://oauth.reddit.com/r/argentina/controversial/.json?count=0&limit=100&t=year&raw_json=1"
        );

        let auth = anonymous::Auth::new();
        let base = auth.base_url();

        let path: PathBuf = ["r", "argentina", "controversial", ".json"]
            .iter()
            .collect();

        let mut params: Vec<(&str, String)> = feed::Options::default().into();

        params.push(("t", feed::TimePeriod::Today.as_str().to_string()));

        assert_eq!(
            build_url(base, &path, &params).to_string(),
            "https://api.reddit.com/r/argentina/controversial/.json?count=0&limit=100&t=day&raw_json=1"
        );
    }
}
