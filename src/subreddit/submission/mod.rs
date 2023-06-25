pub mod stream;

use std::collections::HashMap;

use url::Url;

use crate::response::RedditUrl;

/// [`GalleryData`] contains the data of an item in a Reddit gallery.
#[derive(Debug, serde::Deserialize)]
pub struct GalleryItem {
    /// The gallery item id.
    pub id: i64,
    /// The gallery item media id.
    pub media_id: String,
}

/// [`GalleryData`] contains all items in a Reddit gallery.
#[derive(Debug, serde::Deserialize)]
pub struct Gallery {
    /// The gallery items.
    pub items: Vec<GalleryItem>,
}

/// [`MediaData`]
#[derive(Debug, serde::Deserialize)]
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
#[derive(Debug, serde::Deserialize)]
pub struct MediaData {
    /// The media type.
    pub e: Option<String>,
    /// The media id.
    pub id: String,
    /// The media mime type.
    #[serde(rename = "m")]
    pub mime: String,
    /// The media status.
    pub status: String,
    /// The biggest preview.
    #[serde(rename = "s")]
    pub biggest_preview: Option<MediaProperties>,
}

/// [`RedditVideo`] contains the data of a video that was directly uploaded to Reddit.
#[derive(Debug, serde::Deserialize)]
pub struct RedditVideo {
    /// The video url.
    pub fallback_url: Url,
}

/// [`Media`]
#[derive(Debug, serde::Deserialize)]
pub struct Media {
    /// Where the media comes from.
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    /// The reddit video.
    pub reddit_video: Option<RedditVideo>,
}

#[derive(Debug, serde::Deserialize)]
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
    #[serde(flatten)]
    pub rest: HashMap<String, serde_json::Value>,
}

pub type Submissions = Vec<Submission>;
