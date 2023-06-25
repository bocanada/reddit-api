pub mod feed;
pub mod response;
pub mod submission;

use crate::subreddit::feed::{Options, Sort};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::auth::Authenticator;
use crate::Client;

use self::response::{FeedResponse, SubmissionListing};
use self::submission::stream::SubmissionStreamer;
use self::submission::Submissions;
use crate::response::Generic;

#[derive(Clone)]
pub struct Subreddit<A: Authenticator> {
    pub(crate) client: Client<A>,
    pub name: Arc<str>,
}

impl<A: Authenticator + Send + Sync> Subreddit<A> {
    #[must_use]
    pub fn new(name: &str, client: Client<A>) -> Self {
        Self {
            name: Arc::from(name),
            client,
        }
    }

    /// [`about`] returns information about this [`Subreddit`]
    ///
    /// API Calls to: [`/r/{self.name}/about.json`]
    /// # Errors
    /// Returns `Err` if the underlying [`reqwest::Client::get`] call fails.
    pub async fn about(&self) -> crate::Result<HashMap<String, serde_json::Value>> {
        let path: PathBuf = ["r", &self.name, "about.json"].iter().collect();

        self.client.get_json(&path, &[]).await
    }

    /// [`feed_with_options`] returns submissions sorted by [`Sort`] with [`Options`] on this [`Subreddit`]
    ///
    /// API Calls to: [`/r/{self.name}/{sort}.json`]
    /// # Errors
    /// Returns `Err` if the underlying [`reqwest::Client::get`] call fails.
    pub async fn feed_with_options(
        &self,
        sort: Sort,
        options: Options,
    ) -> crate::Result<SubmissionListing> {
        let path: PathBuf = ["r", &self.name, sort.as_str(), ".json"].iter().collect();
        let mut params: Vec<(&str, String)> = options.into();

        match sort {
            Sort::Top(tp) | Sort::Controversial(tp) => params.push(("t", tp.as_str().to_string())),
            _ => (),
        }

        match self.client.get_json::<FeedResponse>(&path, &params).await? {
            Generic::Listing { data } => Ok(data),
            other => unimplemented!("expected Listing but got {}", other.kind_name()),
        }
    }

    /// [`feed`] returns submissions sorted by [`Sort`] on this [`Subreddit`]
    ///
    /// API Calls to: [`/r/{self.name}/{sort}.json`]
    /// # Errors
    /// Returns `Err` if the underlying [`reqwest::Client::get`] call fails.
    pub async fn feed(&self, sort: Sort) -> crate::Result<Submissions> {
        let t = self.feed_with_options(sort, Options::default()).await?;

        Ok(t.into_iter()
            .map(|c| match c {
                Generic::Link { data } => data,
                other => unreachable!("expected Link but found {}", other.kind_name()),
            })
            .collect())
    }

    /// [`latest`] returns submissions sorted by [`Sort::new`] on this [`Subreddit`]
    ///
    /// API Calls to: [`/r/{self.name}/new.json`]
    /// # Errors
    /// Returns `Err` if the underlying [`reqwest::Client::get`] call fails.
    pub async fn latest(&self) -> crate::Result<Submissions> {
        self.feed(Sort::New).await
    }

    /// [`hot`] returns submissions sorted by [`Sort::hot`] on this [`Subreddit`]
    ///
    /// API Calls to: [`/r/{self.name}/hot.json`]
    /// # Errors
    /// Returns `Err` if the underlying [`reqwest::Client::get`] call fails.
    pub async fn hot(&self) -> crate::Result<Submissions> {
        self.feed(Sort::Hot).await
    }
}

impl<A: Authenticator + Send + Sync + 'static> Subreddit<A> {
    #[must_use]
    pub fn stream_submissions(
        self,
        sort: Sort,
        interval: Duration,
        skip_initial: bool,
    ) -> SubmissionStreamer {
        SubmissionStreamer::new(self, sort, interval, skip_initial)
    }
}

impl<A: Authenticator> std::fmt::Debug for Subreddit<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subreddit")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use dotenv::{dotenv, var};

    use crate::auth::anonymous;
    use crate::errors::Error;
    use crate::subreddit::feed;
    use crate::Client;

    #[tokio::test]
    async fn test_anon_auth() {
        dotenv().unwrap();
        let auth = anonymous::Auth::new();

        let username = var("REDDIT_USERNAME").unwrap();
        let pkg_name = env!("CARGO_PKG_NAME");

        let mut client = Client::new(auth, &format!("{pkg_name} (by u/{username})"));
        assert!(client.login().await.is_ok());

        let sub = client.subreddit("argentina");
        let latest = sub.latest().await;
        assert!(latest.is_ok());

        let latest = sub
            .feed_with_options(
                feed::Sort::Top(feed::TimePeriod::ThisWeek),
                feed::Options::default(),
            )
            .await;
        assert!(latest.is_ok());

        let hot = sub.hot().await;
        assert!(hot.is_ok());

        let sub = client.subreddit("thisdoesntexisttttttttttttttttttt");
        let about = sub.about().await;
        assert!(about.is_err());
        let about = about.unwrap_err();
        assert!(matches!(about, Error::Reddit(..)));
    }
}
