/// [`RedditUrl`] represents a `Url` inside of Reddit.
///
/// This is needed since `Submission.url` may link to another `Submission`, in which case it only contains
/// the path of the `Url`.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum RedditUrl {
    Url(url::Url),
    Permalink(String),
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "kind")]
/// Generic Reddit response, containing all of the valid `kind`s.
pub enum Generic<T> {
    Listing {
        data: ListingKind<T>,
    },
    #[serde(rename = "t3")]
    Link {
        data: T,
    },
    #[serde(rename = "t1")]
    Comment {
        data: T,
    },
    LabeledMulti {
        data: T,
    },
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Listing<T> {
    pub after: Option<String>,
    pub before: Option<String>,
    pub children: Vec<T>,
}

pub type ListingKind<T> = Listing<Generic<T>>;

impl<T> IntoIterator for Listing<T> {
    type Item = T;
    type IntoIter = <std::vec::Vec<T> as std::iter::IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.children.into_iter()
    }
}

impl<T> Generic<T> {
    pub const fn kind_name(&self) -> &str {
        match self {
            Self::Listing { .. } => "listing",
            Self::Link { .. } => "link",
            Self::Comment { .. } => "comment",
            Self::LabeledMulti { .. } => "multi",
        }
    }
}

impl RedditUrl {
    /// Returns a valid [`url::Url`] from this [`RedditUrl`].
    pub fn as_url(&self) -> url::Url {
        match self {
            Self::Url(u) => u.clone(),
            Self::Permalink(perma) => {
                let u = url::Url::parse("https://www.reddit.com/").expect("this to be valid URL");
                u.join(perma).expect("this to be a valid URL")
            }
        }
    }
}

impl From<RedditUrl> for url::Url {
    fn from(value: RedditUrl) -> Self {
        value.as_url()
    }
}

impl std::fmt::Display for RedditUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Url(url) => write!(f, "{}", url.as_str()),
            Self::Permalink(perma) => write!(f, "https://www.reddit.com{perma}"),
        }
    }
}
