pub mod stream;

use std::{collections::HashMap, sync::Arc};

use url::Url;

use crate::response::RedditUrl;

/// [`GalleryData`] contains the data of an item in a Reddit gallery.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GalleryItem {
    /// The gallery item id.
    pub id: i64,
    /// The gallery item media id.
    pub media_id: Arc<str>,
}

/// [`GalleryData`] contains all items in a Reddit gallery.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Gallery {
    /// The gallery items.
    pub items: Arc<[GalleryItem]>,
}

/// [`MediaData`]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MediaProperties {
    #[serde(rename = "u")]
    /// The media url.
    pub url: Option<Url>,
    #[serde(rename = "x")]
    /// The media width.
    pub width: usize,
    #[serde(rename = "y")]
    /// The media height.
    pub height: usize,
}

/// [`MediaData`]
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "e")]
pub enum MediaData {
    RedditVideo {
        /// The media id.
        id: Arc<str>,
        /// The media status.
        status: Status,
        /// The biggest preview.
        #[serde(rename = "s")]
        biggest_preview: Option<MediaProperties>,
    },
    Image {
        /// The media id.
        id: Arc<str>,
        /// The media mime type.
        #[serde(rename = "m")]
        mime: Arc<str>,
        /// The media status.
        status: Status,
        /// The biggest preview.
        #[serde(rename = "s")]
        biggest_preview: Option<MediaProperties>,
    },
    AnimatedImage {
        /// The media id.
        id: Arc<str>,
        /// The media mime type.
        #[serde(rename = "m")]
        mime: Arc<str>,
        /// The media status.
        status: Status,
        /// The biggest preview.
        #[serde(rename = "s")]
        biggest_preview: Option<MediaProperties>,
    },
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
/// Represents the [`MediaData`] [`Status`].
pub enum Status {
    Valid,
    Invalid,
}

/// [`RedditVideo`] contains the data of a video that was directly uploaded to Reddit.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RedditVideo {
    /// The video url.
    pub fallback_url: Url,
}

/// [`Media`]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Media {
    /// Where the media comes from.
    #[serde(rename = "type")]
    pub media_type: Option<Arc<str>>,
    /// The reddit video.
    pub reddit_video: Option<RedditVideo>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Submission {
    /// The author of this post.
    pub author: Arc<str>,
    /// The perma link to this post.
    pub permalink: RedditUrl,
    /// The base36 internal Reddit identifier for this post, e.g. 2qpqw.
    pub id: Arc<str>,
    /// The full 'Thing ID', consisting of a 'kind' and a base-36 identifier. The valid kinds are:
    /// - t1_ - Comment
    /// - t2_ - Account
    /// - t3_ - Link
    /// - t4_ - Message
    /// - t5_ - Subreddit
    /// - t6_ - Award
    /// - t8_ - PromoCampaign
    pub name: Arc<str>,
    /// The linked URL, if this is a link post.
    pub url: Option<RedditUrl>,
    /// The title of the post.
    pub title: Arc<str>,
    /// The subreddit that this submission was posted in (not including `/r/`)
    pub subreddit: Arc<str>,
    /// The items of a gallery.
    pub gallery_data: Option<Gallery>,
    /// The media metadata.
    pub media_metadata: Option<HashMap<Arc<str>, MediaData>>,
    /// This post's media.
    pub media: Option<Media>,
    #[serde(flatten)]
    pub rest: HashMap<Arc<str>, serde_json::Value>,
}

pub type Submissions = Vec<Submission>;
