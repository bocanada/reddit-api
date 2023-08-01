use std::collections::HashMap;

use url::Url;

use crate::response::RedditUrl;

/// [`GalleryItem`] contains the data of an item in a Reddit gallery.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GalleryItem {
    /// The gallery item id.
    pub id: i64,
    /// The gallery item media id.
    pub media_id: String,
}

/// [`Gallery`] contains all items in a Reddit gallery.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Gallery {
    /// The gallery items.
    pub items: Vec<GalleryItem>,
}

/// [`MediaProperties`] contains the media properties of a [`MediaData`]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MediaProperties {
    #[serde(rename = "u")]
    /// The media url.
    pub url: Option<RedditUrl>,
    #[serde(rename = "x")]
    /// The media width.
    pub width: usize,
    #[serde(rename = "y")]
    /// The media height.
    pub height: usize,
}

/// [`MediaData`] contains the media data of a [`Submission`].
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "e")]
pub enum MediaData {
    RedditVideo {
        /// The media id.
        id: String,
        /// The media status.
        status: Status,
        /// The biggest preview.
        #[serde(rename = "s")]
        biggest_preview: Option<MediaProperties>,
    },
    Image {
        /// The media id.
        id: String,
        /// The media mime type.
        #[serde(rename = "m")]
        mime: String,
        /// The media status.
        status: Status,
        /// The biggest preview.
        #[serde(rename = "s")]
        biggest_preview: Option<MediaProperties>,
    },
    AnimatedImage {
        /// The media id.
        id: String,
        /// The media mime type.
        #[serde(rename = "m")]
        mime: String,
        /// The media status.
        status: Status,
        /// The biggest preview.
        #[serde(rename = "s")]
        biggest_preview: Option<MediaProperties>,
    },
}

/// Represents the [`MediaData`] [`Status`].
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
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
    pub media_type: Option<String>,
    /// The reddit video.
    pub reddit_video: Option<RedditVideo>,
}

/// Represents a single [`Submission`].
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Submission {
    /// The author of this post.
    pub author: String,
    /// The perma link to this post.
    pub permalink: RedditUrl,
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
    /// The items of a gallery.
    pub gallery_data: Option<Gallery>,
    /// The media metadata.
    pub media_metadata: Option<HashMap<String, MediaData>>,
    /// This post's media.
    pub media: Option<Media>,
    /// The rest of the attributes as a [`HashMap`].
    #[serde(flatten)]
    pub rest: HashMap<String, serde_json::Value>,
}

/// Represents multiple [`Submission`]s.
pub type Submissions = Vec<Submission>;
