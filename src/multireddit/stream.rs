use std::time::Duration;

use tokio::sync::mpsc::Receiver;

use crate::{
    auth::Authenticator,
    subreddit::{
        feed::Sort,
        submission::{stream::SubmissionStreamer, Submission},
    },
    Stream,
};

use super::Multireddit;

pub struct MultiSubmissionStreamer {
    streams: Vec<SubmissionStreamer>,
    rx: Receiver<crate::Result<Submission>>,
}

impl MultiSubmissionStreamer {
    /// [`MultiSubmissionStreamer::new`] creates a new [`MultiSubmissionStreamer`] instance.
    /// It instantly starts polling the API for data by calling [`Subreddit::feed`] for every
    /// [`Subreddit`] in this [`MultiReddit`] every
    /// [`interval`].
    #[must_use]
    pub fn new<A: Authenticator + 'static>(
        multi: Multireddit<A>,
        sort: Sort,
        interval: Duration,
        skip_initial: bool,
    ) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel((multi.subreddits.len() / 2) * 100);

        let mut streams = Vec::with_capacity(multi.subreddits.len());

        for (i, sub) in multi.subreddits.into_iter().enumerate() {
            let time = Duration::from_millis((60usize + (i * 10)).try_into().unwrap_or_default());
            let stream = SubmissionStreamer::new_with_channels(
                sub,
                sort,
                interval + time,
                skip_initial,
                None,
                tx.clone(),
            );
            streams.push(stream);
        }

        Self { streams, rx }
    }
}

impl Stream for MultiSubmissionStreamer {
    type Item = crate::Result<Submission>;

    /// [`MultiSubmissionStreamer::stop`] stops polling the API.
    /// Keep in mind, [`MultiSubmissionStreamer::next`] may still return [`Some`].
    ///
    /// # Example
    ///
    /// ```
    ///   let pkg_name = env!("CARGO_PKG_NAME");
    ///   let username = env!("REDDIT_USERNAME");
    ///   let mut client = Client::new(
    ///       anonymous::Auth::new(),
    ///       &format!("{pkg_name} (by u/{username})"),
    ///   );
    ///   let sub = client.subreddit("rust");
    ///   let mut stream = sub.stream_submissions(feed::Sort::New, Duration::from_millis(200));
    ///   while let Some(v) = stream.next().await {
    ///       println!("post: {v:?}");
    ///       // Even though we kill it here, stream.next() will return `Some`
    ///       // since the API returns 100 submissions and we stopped it at the first element.
    ///       stream.stop();
    ///   }
    ///
    /// ```
    fn stop(&mut self) {
        for stream in &mut self.streams.iter_mut() {
            stream.stop();
        }
    }
    /// [`MultiSubmissionStreamer::next`] returns the next item in the [`MultiSubmissionStreamer`].
    /// This method will return `None` if [`MultiSubmissionStreamer::stop`] was called and the stream buffer is empty.
    ///
    /// # Example
    ///
    /// ```
    /// let pkg_name = env!("CARGO_PKG_NAME");
    /// let username = env!("REDDIT_USERNAME");
    /// let mut client = Client::new(
    ///     anonymous::Auth::new(),
    ///     &format!("{pkg_name} (by u/{username})"),
    /// );
    /// let multi = client.multi(MultiPath::new("singshredcode", "animal_subbies");
    /// let mut stream = multi.stream_submissions(feed::Sort::New, Duration::from_millis(200));
    ///
    /// while let Some(post) = stream.next().await {
    ///     let post = post?;
    ///
    ///     println!("post: {post:?}");
    /// }
    ///
    /// ```
    async fn next(&mut self) -> Option<crate::Result<Submission>> {
        self.rx.recv().await
    }
}
