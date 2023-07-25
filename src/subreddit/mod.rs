pub mod feed;
pub mod submission;

use crate::subreddit::feed::{Options, Sort};

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::auth::Authenticator;
use crate::Client;
#[cfg(feature = "stream")]
use futures_util::Stream;
#[cfg(feature = "stream")]
use std::collections::HashSet;
#[cfg(feature = "stream")]
use tokio::time::Interval;

use self::submission::Submission;
use self::submission::Submissions;
use crate::response::Generic;

type FeedResponse = Generic<Submission>;

#[derive(Clone)]
pub struct Subreddit<A: Authenticator> {
    pub(crate) client: Client<A>,
    pub name: Arc<str>,
}

impl<A> Subreddit<A>
where
    A: Authenticator,
{
    #[must_use]
    pub fn new(name: &str, client: Client<A>) -> Self {
        Self {
            name: Arc::from(name),
            client,
        }
    }

    /// [`Subreddit::about`] returns information about this [`Subreddit`]
    ///
    /// API Calls to: [`/r/{self.name}/about.json`]
    /// # Errors
    /// Returns `Err` if the underlying [`reqwest::Client::get`] call fails.
    pub async fn about(&self) -> crate::Result<HashMap<String, serde_json::Value>> {
        let path: PathBuf = ["r", &self.name, "about.json"].iter().collect();

        self.client.get_json(&path, &[]).await
    }

    /// [`Subreddit::feed_with_options`] returns submissions sorted by [`Sort`] with [`Options`] on this [`Subreddit`]
    ///
    /// API Calls to: [`/r/{self.name}/{sort}.json`]
    /// # Errors
    /// Returns `Err` if the underlying [`reqwest::Client::get`] call fails.
    pub async fn feed_with_options(
        &self,
        sort: Sort,
        options: Options,
    ) -> crate::Result<Submissions> {
        let path: PathBuf = ["r", &self.name, sort.as_str(), ".json"].iter().collect();
        let mut params: Vec<(&str, String)> = options.into();

        match sort {
            Sort::Top(tp) | Sort::Controversial(tp) => params.push(("t", tp.as_str().to_string())),
            _ => (),
        }

        match self.client.get_json::<FeedResponse>(&path, &params).await? {
            Generic::Listing { data } => Ok(data
                .into_iter()
                .map(|c| match c {
                    Generic::Link { data } => data,
                    other => unimplemented!("expected Listing but got {}", other.kind_name()),
                })
                .collect()),
            other => unimplemented!("expected Listing but got {}", other.kind_name()),
        }
    }

    /// [`Subreddit::feed`] returns submissions sorted by [`Sort`] on this [`Subreddit`]
    ///
    /// API Calls to: [`/r/{self.name}/{sort}.json`]
    /// # Errors
    /// Returns `Err` if the underlying [`reqwest::Client::get`] call fails.
    pub async fn feed(&self, sort: Sort) -> crate::Result<Submissions> {
        self.feed_with_options(sort, Options::default()).await
    }

    /// [`Subreddit::latest`] returns submissions sorted by [`Sort::New`] on this [`Subreddit`]
    ///
    /// API Calls to: [`/r/{self.name}/new.json`]
    /// # Errors
    /// Returns `Err` if the underlying [`reqwest::Client::get`] call fails.
    pub async fn latest(&self) -> crate::Result<Submissions> {
        self.feed(Sort::New).await
    }

    /// [`Subreddit::hot`] returns submissions sorted by [`Sort::Hot`] on this [`Subreddit`]
    ///
    /// API Calls to: [`/r/{self.name}/hot.json`]
    /// # Errors
    /// Returns `Err` if the underlying [`reqwest::Client::get`] call fails.
    pub async fn hot(&self) -> crate::Result<Submissions> {
        self.feed(Sort::Hot).await
    }

    /// Creates a new [`Stream`] of [`Submission`].
    /// If `tick_first` is set, it first waits for the interval to run before calling the API.
    #[cfg(feature = "stream")]
    pub(crate) fn stream_inner(
        self,
        sort: Sort,
        interval: Interval,
        tick_first: bool,
    ) -> impl Stream<Item = crate::Result<Submission>> + Unpin {
        Box::pin(futures_util::stream::unfold(
            (
                self,
                interval,
                Vec::with_capacity(100),
                HashSet::with_capacity(100),
                tick_first,
            ),
            move |(this, mut interval, mut queue, mut set, tick_first)| async move {
                if tick_first {
                    interval.tick().await;
                }

                loop {
                    if let Some(post) = queue.pop().map(Ok) {
                        return Some((post, (this, interval, queue, set, false)));
                    }
                    interval.tick().await;
                    match this.feed(sort).await {
                        Err(e) => return Some((Err(e), (this, interval, queue, set, false))),
                        Ok(posts) => {
                            queue.extend(posts.into_iter().filter(|p| set.insert(p.id.clone())));
                            continue;
                        }
                    }
                }
            },
        ))
    }

    /// Creates a new [`Stream`] of [`Submission`].
    #[cfg(feature = "stream")]
    #[doc(cfg(feature = "stream"))]
    pub fn stream(
        self,
        sort: Sort,
        interval: Interval,
    ) -> impl Stream<Item = crate::Result<Submission>> + Unpin {
        self.stream_inner(sort, interval, false)
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

    use crate::errors::Error;
    use crate::subreddit::feed;
    use crate::Client;

    #[cfg(feature = "stream")]
    use futures_util::StreamExt;
    #[cfg(feature = "stream")]
    use tokio::time::interval;

    #[cfg(feature = "stream")]
    #[tokio::test]
    async fn test_fut_stream() {
        use std::time::Duration;

        dotenv().unwrap();
        let username = var("REDDIT_USERNAME").unwrap();
        let pkg_name = env!("CARGO_PKG_NAME");

        let client = Client::new(&format!("{pkg_name} (by u/{username})"));

        let sub = client.subreddit("argentina");
        let stream = sub
            .stream(feed::Sort::New, interval(Duration::from_secs(200)))
            .take(100)
            .fold(0, |state, _| async move { state + 1 })
            .await;

        assert_eq!(stream, 100);
    }
    #[tokio::test]
    async fn test_sub_feed() {
        dotenv().unwrap();
        let username = var("REDDIT_USERNAME").unwrap();
        let pkg_name = env!("CARGO_PKG_NAME");

        let client = Client::new(&format!("{pkg_name} (by u/{username})"));

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
