use crate::{
    auth::Authenticator,
    response::{Generic, RedditUrl},
    subreddit::Subreddit,
    Client,
};

use url::Url;

use super::Multireddit;

#[derive(Debug, serde::Deserialize)]
pub struct SubredditMeta {
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct MultiInternal {
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
    pub subreddits: Vec<SubredditMeta>,
}

impl MultiInternal {
    pub(crate) fn into_usable<A: Authenticator>(self, client: &Client<A>) -> Multireddit<A> {
        Multireddit {
            can_edit: self.can_edit,
            created: self.created,
            created_utc: self.created_utc,
            description_html: self.description_html,
            description_md: self.description_md,
            display_name: self.display_name,
            icon_url: self.icon_url,
            key_color: self.key_color,
            name: self.name,
            num_subscribers: self.num_subscribers,
            over_18: self.over_18,
            owner: self.owner,
            owner_id: self.owner_id,
            path: self.path,
            subreddits: self
                .subreddits
                .into_iter()
                .map(|s| Subreddit::new(&s.name, client.clone()))
                .collect(),
        }
    }
}

#[allow(clippy::pedantic)]
pub type MultiResponse = Generic<MultiInternal>;
