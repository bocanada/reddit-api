use futures_util::{stream::SelectAll, Stream};

use crate::auth::Authenticator;

use super::submission::Submission;
use super::{feed, Subreddit};
use futures_util::stream::select_all;
use nanorand::{Rng, WyRand};
use std::collections::HashSet;
use std::{ops::RangeBounds, time::Duration};
use tokio::time::{interval, Interval};

pub type MultiStream<T> = SelectAll<T>;
type Subreddits<A> = Vec<Subreddit<A>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy, thiserror::Error)]
pub enum Error {
    #[error("there are no subreddits to stream from")]
    MissingSubreddits,
    #[error("no poll period was set")]
    MissingPollPeriod,
}

/// Builds a [`Stream`] of [`Submission`]s.
///
/// # Example
/// ```
/// use reddit_api::{Stream, submission::StreamBuilder};
///
/// # fn main {
///    let mut stream = StreamBuilder::new()
///        .poll_period(Duration::from_secs(60))
///        .add_sub(client.subreddit("kpop"))
///        .skip_initial(false)
///        .build(0..0)
///        .unwrap()
///        .take(20);
///
///    while let Some(res) = stream.next().await {
///        eprintln!("{:#?}", res);
///    }
///
/// # }
/// ```
pub struct StreamBuilder<A>
where
    A: Authenticator,
{
    skip_initial: bool,
    subreddits: Subreddits<A>,
    sort: feed::Sort,
    period: Option<Duration>,
}

impl<A> StreamBuilder<A>
where
    A: Authenticator,
{
    /// Creates a new [`StreamBuilder`] instance.
    #[must_use = "builder does nothing unless built"]
    pub const fn new() -> Self {
        Self {
            skip_initial: true,
            period: None,
            subreddits: Vec::new(),
            sort: feed::Sort::New,
        }
    }

    /// Adds a [`Subreddit`] from where to pull [`Submission`]s from.
    #[must_use = "builder does nothing unless built"]
    pub fn add_sub(mut self, sub: Subreddit<A>) -> Self {
        self.subreddits.push(sub);
        self
    }

    /// Adds multiple [`Subreddit`]s from where to pull [`Submission`]s from.
    #[must_use = "builder does nothing unless built"]
    pub fn add_subs<I>(mut self, subs: I) -> Self
    where
        I: IntoIterator<Item = Subreddit<A>>,
    {
        self.subreddits.extend(subs);
        self
    }

    /// Sets the [`feed::Sort`] order of the [`Submission`] feed.
    #[must_use = "builder does nothing unless built"]
    pub const fn sort(mut self, sort: feed::Sort) -> Self {
        self.sort = sort;
        self
    }

    /// Skips initial [`Submission`]s.
    #[must_use = "builder does nothing unless built"]
    pub const fn skip_initial(mut self, skip: bool) -> Self {
        self.skip_initial = skip;
        self
    }

    /// Sets the wait time in between polls.
    #[must_use = "builder does nothing unless built"]
    pub const fn poll_period(mut self, period: Duration) -> Self {
        self.period = Some(period);
        self
    }

    /// Builds the [`Stream`].
    ///
    /// # Errors
    /// This function fails if no [`Subreddit`] was added to the [`StreamBuilder`] or if the
    /// [`StreamBuilder#period`] was not set.
    pub fn build<B>(
        self,
        spread: B,
    ) -> Result<MultiStream<impl Stream<Item = crate::Result<Submission>>>, Error>
    where
        B: RangeBounds<u64> + Clone,
    {
        if self.subreddits.is_empty() {
            return Err(Error::MissingSubreddits);
        }

        let period = self.period.ok_or(Error::MissingPollPeriod)?;

        let mut rng = WyRand::new();

        let should_tick_rand = self.subreddits.len() > 1;

        Ok(select_all(self.subreddits.into_iter().map(|sub| {
            let range = spread.clone();
            let dur = period + Duration::from_secs(rng.generate_range(range));
            let state = StreamState::new(
                self.skip_initial,
                if should_tick_rand {
                    rng.generate()
                } else {
                    false
                },
                self.sort,
                interval(dur),
            );
            sub.stream_inner(state)
        })))
    }
}

#[derive(Debug)]
pub struct StreamState {
    pub skip_initial: bool,
    pub tick_first: bool,
    pub sort: feed::Sort,
    pub every: Interval,
    pub queue: Vec<Submission>,
    pub seen: HashSet<String>,
}

impl StreamState {
    #[must_use]
    pub fn new(skip_initial: bool, tick_first: bool, sort: feed::Sort, every: Interval) -> Self {
        Self {
            skip_initial,
            tick_first,
            sort,
            every,
            queue: Vec::with_capacity(100),
            seen: HashSet::with_capacity(100),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{auth::Anon, Client};

    use super::StreamBuilder;

    #[tokio::test]
    async fn test_poll_period_missing() {
        let client = Client::new("test by git/Bocanada");
        let b = StreamBuilder::new()
            .add_sub(client.subreddit("kpop"))
            .skip_initial(false)
            .build(0..0);

        assert!(b.is_err());
        let Err(err) = b else { unreachable!() };
        assert_eq!(err, super::Error::MissingPollPeriod);
    }

    #[tokio::test]
    async fn test_subreddit_missing() {
        let b = StreamBuilder::<Anon>::new()
            .skip_initial(false)
            .poll_period(Duration::from_secs(60))
            .build(0..0);

        assert!(b.is_err());
        let Err(err) = b else { unreachable!() };
        assert_eq!(err, super::Error::MissingSubreddits);
    }
}
