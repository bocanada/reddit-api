use crate::{
    response::{Generic, Listing},
    subreddit::submission::Submission,
};

#[allow(clippy::pedantic)]
pub type FeedResponse = Generic<Submission>;

pub type SubmissionListing = Listing<Generic<Submission>>;
