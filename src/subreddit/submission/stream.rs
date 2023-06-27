use std::time::Duration;
use std::{collections::HashSet, sync::Arc};

use super::Submission;
use crate::{
    auth::Authenticator,
    subreddit::{feed::Sort, Subreddit},
    Stream,
};
use tokio::time::{interval, Interval};

#[derive(Debug)]
pub struct SubmissionStreamer<A: Authenticator> {
    sub: Subreddit<A>,
    sort: Sort,

    interval: Interval,

    skip_initial: bool,
    is_stopped: bool,

    /// This queue is only going to ever be built of [`Submission`]s we haven't already seen.
    queue: Vec<Submission>,
    seen: HashSet<Arc<str>>,
}

impl<A: Authenticator> SubmissionStreamer<A> {
    /// [`SubmissionStream::new`] creates a new [`SubmissionStream`] instance.
    /// It instantly starts polling the API for data by calling [`Subreddit::feed`] every
    /// [`interval`].
    #[must_use]
    pub fn new(
        sub: Subreddit<A>,
        sort: Sort,
        interval_period: Duration,
        skip_initial: bool,
    ) -> Self {
        let mut interval = interval(interval_period);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        Self {
            sub,
            sort,
            interval,
            queue: Vec::with_capacity(100),
            seen: HashSet::with_capacity(100),
            is_stopped: false,
            skip_initial,
        }
    }
}

impl<A: Authenticator> Stream for SubmissionStreamer<A> {
    type Item = crate::Result<Submission>;

    /// [`SubmissionStream::stop`] stops polling the API.
    /// Keep in mind, [`SubmissionStream::next`] may still return [`Some`].
    ///
    /// # Example
    ///
    /// ```
    ///   let pkg_name = env!("CARGO_PKG_NAME");
    ///   let username = env!("REDDIT_USERNAME");
    ///   let mut client = Client::new(
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
        self.is_stopped = true;
    }

    /// [`SubmissionStream::next`] returns the next item in the [`SubmissionStream`].
    /// This method will return `None` if [`SubmissionStream::stop`] was called and the stream buffer is empty.
    ///
    /// # Example
    ///
    /// ```
    /// let pkg_name = env!("CARGO_PKG_NAME");
    /// let username = env!("REDDIT_USERNAME");
    /// let mut client = Client::new(
    ///     &format!("{pkg_name} (by u/{username})"),
    /// );
    /// let sub = client.subreddit("rust");
    /// let mut stream = sub.stream_submissions(feed::Sort::New, Duration::from_millis(200));
    ///
    /// while let Some(post) = stream.next().await {
    ///     let post = post?;
    ///
    ///     println!("post: {post:?}");
    /// }
    ///
    /// ```
    async fn next(&mut self) -> Option<Self::Item> {
        if let Some(post) = self.queue.pop().map(Ok) {
            return Some(post);
        }

        // If we got here, the queue is empty.
        // Loop until we get some new posts or self was stopped by calling [`Stream::stop`].
        while !self.is_stopped {
            self.interval.tick().await;

            match self.sub.feed(self.sort).await {
                Ok(posts) => {
                    if self.skip_initial {
                        self.seen.extend(posts.into_iter().map(|p| p.id));
                        self.skip_initial = false;
                        continue;
                    }

                    self.queue
                        // Filter out the already seen values
                        .extend(posts.into_iter().filter(|p| self.seen.insert(p.id.clone())));

                    if let Some(post) = self.queue.pop().map(Ok) {
                        return Some(post);
                    }
                    continue;
                }
                Err(e) => return Some(Err(e)),
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use dotenv::{dotenv, var};

    use crate::subreddit::feed;
    use crate::Client;
    use crate::Stream;
    use std::time::Duration;

    #[tokio::test]
    async fn test_stream() {
        dotenv().unwrap();
        let username = var("REDDIT_USERNAME").unwrap();
        let pkg_name = env!("CARGO_PKG_NAME");
        let user_agent = format!("{pkg_name} (by u/{username})");

        let client = Client::new(&user_agent);

        let sub = client.subreddit("rust");
        let mut rust_stream =
            sub.stream_submissions(feed::Sort::New, Duration::from_millis(200), false);

        let sub = client.subreddit("python");
        let mut py_stream =
            sub.stream_submissions(feed::Sort::New, Duration::from_millis(200), false);

        let mut i = 0;

        loop {
            if i == 200 {
                break;
            }
            let res = tokio::select! {
                Some(res) = rust_stream.next() => {
                    rust_stream.stop();
                    res
                },
                Some(res) = py_stream.next() => {
                    py_stream.stop();
                    res
                }
            };
            assert!(res.is_ok());

            i += 1;
        }
    }
}
