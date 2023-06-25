pub mod stream;

use std::collections::HashMap;

use crate::response::RedditUrl;

#[derive(Debug, serde::Deserialize)]
pub struct Submission {
    /// The author of this post.
    pub author: String,
    /// The path to this post.
    pub permalink: String,
    /// The base36 internal Reddit identifier for this post, e.g. 2qpqw.
    pub id: String,
    /// The full 'Thing ID', consisting of a 'kind' and a base-36 identifier. The valid kinds are:
    /// - t1_ - Comment
    /// - t2_ - Account
    /// - t3_ - Link
    /// - t4_ - Message
    /// - t5_ - Subreddit
    /// - t6_ - Award
    /// - t8_ - PromoCampaign
    pub name: String,
    /// The linked URL, if this is a link post.
    pub url: Option<RedditUrl>,
    /// The title of the post.
    pub title: String,
    /// The subreddit that this submission was posted in (not including `/r/`)
    pub subreddit: String,
    #[serde(flatten)]
    pub rest: HashMap<String, serde_json::Value>,
}

pub type Submissions = Vec<Submission>;
