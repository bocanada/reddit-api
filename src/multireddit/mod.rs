pub mod response;

use url::Url;

use crate::response::RedditUrl;
use crate::subreddit::Subreddit;
use std::path::PathBuf;
use std::sync::Arc;

use crate::auth::Authenticator;

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
    pub subreddits: Arc<[Subreddit<A>]>,
}

// TODO: Implement stream for MultiReddit

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

    use dotenv::{dotenv, var};

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
}
