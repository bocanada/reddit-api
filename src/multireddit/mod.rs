pub mod response;
pub mod stream;

use url::Url;

use crate::response::RedditUrl;
use crate::subreddit::feed::Sort;
use crate::subreddit::Subreddit;
use std::sync::Arc;
use std::{path::PathBuf, time::Duration};

use crate::auth::Authenticator;

use self::stream::MultiSubmissionStreamer;

#[derive(Clone, Debug)]
pub struct Multireddit<A: Authenticator> {
    pub can_edit: bool,
    pub created: f64,
    pub created_utc: f64,
    pub description_html: String,
    pub description_md: String,
    pub display_name: String,
    pub icon_url: Url,
    pub key_color: String,
    pub name: String,
    pub num_subscribers: u64,
    pub over_18: bool,
    pub owner: String,
    pub owner_id: String,
    pub path: RedditUrl,
    pub subreddits: Vec<Subreddit<A>>,
}

impl<A: Authenticator + Send + Sync + 'static> Multireddit<A> {
    /// # Errors
    /// Returns [`Err`] if the call to [`Multireddit::fetch`] fails.
    #[must_use]
    pub fn stream_submissions(
        self,
        sort: Sort,
        interval: Duration,
        skip_initial: bool,
    ) -> MultiSubmissionStreamer {
        MultiSubmissionStreamer::new(self, sort, interval, skip_initial)
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
    use std::path::PathBuf;
    use std::time::Duration;

    use dotenv::{dotenv, var};

    use super::MultiPath;
    use crate::subreddit::feed::Sort;
    use crate::Client;
    use crate::Stream;

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
    async fn anon_multi_user_stream() {
        let pkg_name = env!("CARGO_PKG_NAME");
        let username = var("REDDIT_USERNAME").unwrap();

        let client = Client::new(&format!("{pkg_name} (by u/{username})"));

        let multi = client
            .multi(MultiPath::new("singshredcode", "animal_subbies"))
            .await;

        assert!(multi.is_ok());
        let multi = multi.unwrap();

        // should be: multi.subreddits.len() * 100;
        // but w/e
        let expects = multi.subreddits.len() * 50;
        let mut i = 0;
        let mut stream = multi.stream_submissions(Sort::New, Duration::from_secs(60), false);

        while let Some(post) = stream.next().await {
            if i >= expects {
                stream.stop();
                break;
            }
            assert!(post.is_ok());
            i += 1;
        }
    }
}
