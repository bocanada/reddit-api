use std::collections::HashSet;
use std::time::Duration;

use tokio::{
    sync::mpsc::Receiver,
    sync::mpsc::{error::SendError, Sender},
    task::JoinHandle,
};

use crate::{
    auth::Authenticator,
    subreddit::{feed::Sort, Subreddit},
    Stream,
};

use super::Submission;

#[derive(Debug)]
pub struct SubmissionStreamer {
    jh: JoinHandle<Result<(), SendError<crate::Result<Submission>>>>,
    rx: Receiver<crate::Result<Submission>>,
}

impl SubmissionStreamer {
    /// [`SubmissionStream::new`] creates a new [`SubmissionStream`] instance.
    /// It instantly starts polling the API for data by calling [`Subreddit::feed`] every
    /// [`interval`].
    #[must_use]
    pub fn new<A: Authenticator + 'static>(
        subreddit: Subreddit<A>,
        sort: Sort,
        interval: Duration,
        skip_initial: bool,
    ) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        Self::new_with_channels(subreddit, sort, interval, skip_initial, Some(rx), tx)
    }

    pub(crate) fn new_with_channels<A: Authenticator + Send + Sync + 'static>(
        subreddit: Subreddit<A>,
        sort: Sort,
        interval: Duration,
        skip_initial: bool,
        rx: Option<Receiver<crate::Result<Submission>>>,
        tx: Sender<crate::Result<Submission>>,
    ) -> Self {
        let rx = rx.map_or_else(|| tokio::sync::mpsc::channel(1).1, |rx| rx);

        let jh: JoinHandle<Result<(), SendError<crate::Result<Submission>>>> = tokio::spawn({
            let mut every = tokio::time::interval(interval);
            every.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            async move {
                let sub = subreddit;

                let mut seen: HashSet<std::sync::Arc<str>> = if skip_initial {
                    let set = sub.feed(sort).await.map_or_else(
                        |_| HashSet::with_capacity(100),
                        |items| items.into_iter().map(|s| s.id).collect(),
                    );
                    // This completes immediatly
                    every.tick().await;
                    set
                } else {
                    HashSet::with_capacity(100)
                };

                loop {
                    every.tick().await;

                    match sub.feed(sort).await {
                        Ok(submissions) => {
                            for submission in submissions {
                                if seen.contains(&submission.id) {
                                    continue;
                                }
                                seen.insert(submission.id.clone());
                                tx.send(Ok(submission)).await?;
                            }
                        }
                        Err(e) => {
                            tx.send(Err(e)).await?;
                        }
                    }
                }
            }
        });

        Self { jh, rx }
    }
}

impl Stream for SubmissionStreamer {
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
        self.jh.abort();
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
    async fn next(&mut self) -> Option<crate::Result<Submission>> {
        self.rx.recv().await
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
