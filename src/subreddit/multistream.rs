use futures_util::{stream::SelectAll, Stream};
use sqlx::SqlitePool;

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
    Subreddits,
    #[error("no poll period was set")]
    PollPeriod,
    #[error("no storage was set")]
    Storage,
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
#[derive(Default)]
pub struct StreamBuilder<A, S>
where
    A: Authenticator,
{
    skip_initial: bool,
    subreddits: Subreddits<A>,
    sort: feed::Sort,
    period: Option<Duration>,
    storage: Option<S>,
}

impl<A, S> StreamBuilder<A, S>
where
    A: Authenticator,
    S: Storage + Clone,
{
    /// Creates a new [`StreamBuilder`] instance.
    #[must_use = "builder does nothing unless built"]
    pub const fn new() -> Self {
        Self {
            skip_initial: true,
            period: None,
            subreddits: Vec::new(),
            sort: feed::Sort::New,
            storage: None,
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

    /// Sets a [`Storage`] to save seen [`Submission`]s from this [`Subreddit`].
    #[must_use = "builder does nothing unless built"]
    pub fn set_storage(mut self, storage: S) -> Self {
        self.storage = Some(storage);
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
            return Err(Error::Subreddits);
        }

        let period = self.period.ok_or(Error::PollPeriod)?;
        let storage = self.storage.ok_or(Error::Storage)?;

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
                storage.clone(),
            );
            sub.stream_inner(state)
        })))
    }
}

#[derive(Debug)]
pub struct StreamState<S> {
    pub skip_initial: bool,
    pub tick_first: bool,
    pub sort: feed::Sort,
    pub every: Interval,
    pub queue: Vec<Submission>,
    pub seen: S,
}

impl<S> StreamState<S>
where
    S: Storage,
{
    #[must_use]
    pub fn new(
        skip_initial: bool,
        tick_first: bool,
        sort: feed::Sort,
        every: Interval,
        storage: S,
    ) -> Self {
        Self {
            skip_initial,
            tick_first,
            sort,
            every,
            queue: Vec::with_capacity(100),
            seen: storage,
        }
    }
}

pub trait Storage {
    /// # Returns
    /// Wether the submission was seen or not.
    fn store(&mut self, sub: &Submission)
        -> impl std::future::Future<Output = crate::Result<bool>>;

    fn store_all<'a, I: IntoIterator<Item = &'a Submission>>(
        &mut self,
        it: I,
    ) -> impl std::future::Future<Output = crate::Result<()>> {
        async {
            for e in it {
                self.store(e).await?;
            }

            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetStorage(HashSet<String>);

impl SetStorage {
    #[must_use]
    pub fn new() -> Self {
        Self(HashSet::with_capacity(100))
    }
}

impl Storage for SetStorage {
    async fn store(&mut self, sub: &Submission) -> crate::Result<bool> {
        Ok(self.0.insert(sub.id.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct SqliteStorage(SqlitePool);

impl SqliteStorage {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self(pool)
    }

    /// Call this function to create the necessary tables to Store the reddit API data.
    pub async fn init(&self) -> crate::Result<()> {
        // the primary key is the row id...
        sqlx::query!(
            r#"CREATE TABLE IF NOT EXISTS post (
                id TEXT  NOT NULL,
                sub TEXT NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_DATE,
                CONSTRAINT u_id_sub UNIQUE (id, sub)
            )"#
        )
        .execute(&self.0)
        .await?;

        Ok(())
    }
}

impl Storage for SqliteStorage {
    async fn store(&mut self, sub: &Submission) -> crate::Result<bool> {
        let rows_affected = sqlx::query!(
            "INSERT OR IGNORE INTO post(id, sub) VALUES (?, ?)",
            sub.id,
            sub.subreddit,
        )
        .execute(&self.0)
        .await?;

        Ok(rows_affected.rows_affected() == 1)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{auth::Anon, subreddit::multistream::SetStorage, Client};

    use super::StreamBuilder;

    #[tokio::test]
    async fn test_poll_period_missing() {
        let client = Client::new("test by git/Bocanada");
        let b = StreamBuilder::new()
            .add_sub(client.subreddit("kpop"))
            .set_storage(SetStorage::new())
            .skip_initial(false)
            .build(0..0);

        assert!(b.is_err());
        let Err(err) = b else { unreachable!() };
        assert_eq!(err, super::Error::PollPeriod);
    }

    #[tokio::test]
    async fn test_subreddit_missing() {
        let b = StreamBuilder::<Anon, _>::new()
            .skip_initial(false)
            .poll_period(Duration::from_secs(60))
            .set_storage(SetStorage::new())
            .build(0..0);

        assert!(b.is_err());
        let Err(err) = b else { unreachable!() };
        assert_eq!(err, super::Error::Subreddits);
    }
}
