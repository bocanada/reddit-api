pub(crate) mod response;

use url::Url;

use crate::response::RedditUrl;
#[cfg(feature = "stream")]
use crate::subreddit::submission::Submission;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::auth::Authenticator;
#[cfg(feature = "stream")]
use crate::subreddit::feed;
use crate::subreddit::Subreddit;
#[cfg(feature = "stream")]
use futures_util::{stream::SelectAll, Stream};
#[cfg(feature = "stream")]
use std::ops::RangeBounds;

#[derive(Clone, Debug)]
pub struct Multireddit<A: Authenticator> {
    pub can_edit: bool,
    pub created: f64,
    pub created_utc: f64,
    pub description_html: Arc<str>,
    pub description_md: Arc<str>,
    pub display_name: Arc<str>,
    pub icon_url: Url,
    pub key_color: Arc<str>,
    pub name: Arc<str>,
    pub num_subscribers: u64,
    pub over_18: bool,
    pub owner: Arc<str>,
    pub owner_id: Arc<str>,
    pub path: RedditUrl,
    pub subreddits: Vec<Subreddit<A>>,
}

#[cfg(feature = "stream")]
type MultiStream<T> = SelectAll<T>;

impl<A: Authenticator> Multireddit<A> {
    /// Creates a new [`Stream`] of [`Submission`].
    #[cfg(feature = "stream")]
    #[doc(cfg(feature = "stream"))]
    pub fn stream(
        self,
        sort: feed::Sort,
        every: Duration,
        spread: impl RangeBounds<u64> + Clone,
    ) -> MultiStream<impl Stream<Item = crate::Result<Submission>>> {
        use futures_util::stream::select_all;
        use nanorand::{Rng, WyRand};
        use tokio::time::interval;
        let mut rng = WyRand::new();

        select_all(self.subreddits.into_iter().map(|sub| {
            let every = interval(every + Duration::from_secs(rng.generate_range(spread.clone())));
            sub.stream_inner(sort, every, rng.generate())
        }))
    }
}

#[derive(Debug, Clone)]
pub struct MultiPath {
    /// A User multi, (username, multi name)
    username: Arc<str>,
    name: Arc<str>,
}

impl MultiPath {
    #[must_use]
    pub fn new(username: &str, name: &str) -> Self {
        Self {
            username: Arc::from(username),
            name: Arc::from(name),
        }
    }
}

impl From<&MultiPath> for PathBuf {
    fn from(value: &MultiPath) -> Self {
        let mut starting: Self = "api/multi".into();

        starting.extend(&["user", &value.username, "m", &value.name]);
        starting
    }
}

impl From<MultiPath> for PathBuf {
    fn from(value: MultiPath) -> Self {
        (&value).into()
    }
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, time::Duration};

    use dotenv::{dotenv, var};
    use futures_util::StreamExt;

    use super::MultiPath;
    use crate::Client;

    #[test]
    fn multipath_to_pathbuf() {
        let a: PathBuf = MultiPath::new("singshredcode", "animal_subbies").into();
        assert_eq!(
            a.to_str().unwrap(),
            "api/multi/user/singshredcode/m/animal_subbies"
        );
    }

    #[tokio::test]
    async fn anon_multi_user() {
        dotenv().unwrap();
        let pkg_name = env!("CARGO_PKG_NAME");
        let username = var("REDDIT_USERNAME").unwrap();

        let client = Client::new(&format!("{pkg_name} (by u/{username})"));

        let multi = client
            .multi(MultiPath::new("singshredcode", "animal_subbies"))
            .await;
        assert!(multi.is_ok());
    }

    #[tokio::test]
    async fn anon_multi_stream() {
        dotenv().unwrap();
        let pkg_name = env!("CARGO_PKG_NAME");
        let username = var("REDDIT_USERNAME").unwrap();

        let client = Client::new(&format!("{pkg_name} (by u/{username})"));

        let multi = client
            .multi(MultiPath::new("singshredcode", "animal_subbies"))
            .await;
        assert!(multi.is_ok());
        let multi = multi.unwrap();
        let n = multi
            .stream(
                crate::subreddit::feed::Sort::New,
                Duration::from_secs(120),
                20..=60,
            )
            .take(200)
            .take_while(|r| futures_util::future::ready(r.is_ok()))
            .fold(0, |state, next| async move {
                next.map(|_| 1).unwrap_or(0) + state
            })
            .await;

        assert_eq!(n, 200);
    }
}
