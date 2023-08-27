pub(crate) mod response;

use url::Url;

use crate::response::RedditUrl;
#[cfg(feature = "stream")]
use crate::subreddit::multistream::StreamBuilder;
use std::path::PathBuf;
use std::sync::Arc;

use crate::auth::Authenticator;
use crate::subreddit::Subreddit;

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
impl<A: Authenticator> Multireddit<A> {
    /// Creates a new [`StreamBuilder`] with all the [`Subreddit`] added.
    #[doc(cfg(feature = "stream"))]
    #[must_use = "builder does nothing unless built"]
    pub fn stream(self) -> StreamBuilder<A> {
        StreamBuilder::new().add_subs(self.subreddits)
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

        let multi = dbg!(
            client
                .multi(MultiPath::new("singshredcode", "animal_subbies"))
                .await
        );
        assert!(multi.is_ok());
        let multi = multi.unwrap();
        let n = multi
            .stream()
            .sort(crate::subreddit::feed::Sort::New)
            .poll_period(Duration::from_secs(120))
            .build(30..=120)
            .unwrap()
            .take(200)
            .take_while(|r| futures_util::future::ready(r.is_ok()))
            .fold(0, |state, next| async move {
                next.map(|_| 1).unwrap_or(0) + state
            })
            .await;

        assert_eq!(n, 200);
    }
}
