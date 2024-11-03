use std::sync::Arc;

/// [`Options`] for calling the Reddit API.
#[derive(Clone, Debug)]
pub struct Options {
    /// `after` indicates the fullname of an item in the listing to use as the anchor point of the slice.
    after: Option<Arc<str>>,

    /// `before` indicates the fullname of an item in the listing to use as the anchor point of the slice.
    before: Option<Arc<str>>,

    /// The number of items already seen in this listing.
    count: u64,

    /// The number of items that can be in this listing.
    limit: u64,
}

/// Allows you to request submissions by a `Sort`.
#[derive(Copy, Clone, Debug, Default)]
pub enum Sort {
    /// Top posts by `TimePeriod`
    Top(TimePeriod),
    /// Controversial posts by `TimePeriod`
    Controversial(TimePeriod),
    /// Hot posts
    Hot,
    /// Rising posts
    Rising,
    /// New posts
    #[default]
    New,
}

/// Allows you to request a certain time period. This only works in certain situations, like when asking for top of a subreddit
#[derive(Copy, Clone, Debug)]
pub enum TimePeriod {
    /// Posts from very recently
    Now,
    /// Posts from today
    Today,
    /// Posts from this week
    ThisWeek,
    /// Posts from this month
    ThisMonth,
    /// Posts from this year
    ThisYear,
    /// All posts
    AllTime,
}

impl Sort {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Top(_) => "top",
            Self::Controversial(_) => "controversial",
            Self::Hot => "hot",
            Self::Rising => "rising",
            Self::New => "new",
        }
    }
}

impl TimePeriod {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Now => "hour",
            Self::Today => "day",
            Self::ThisWeek => "week",
            Self::ThisMonth => "month",
            Self::ThisYear => "year",
            Self::AllTime => "all",
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            after: None,
            before: None,
            count: 0,
            limit: 100,
        }
    }
}

impl Options {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn after(mut self, after: &str) -> Self {
        self.after = Some(Arc::from(after));
        self
    }

    #[must_use]
    pub fn before(mut self, before: &str) -> Self {
        self.before = Some(Arc::from(before));
        self
    }

    #[must_use]
    pub const fn count(mut self, count: u64) -> Self {
        self.count = count;
        self
    }

    #[must_use]
    pub const fn limit(mut self, limit: u64) -> Self {
        self.limit = limit;
        self
    }
}

impl From<Options> for Vec<(&str, String)> {
    fn from(value: Options) -> Self {
        let mut params = Vec::with_capacity(4);

        params.extend([
            ("count", value.count.to_string()),
            ("limit", value.limit.to_string()),
        ]);

        if let Some(after) = value.after {
            params.push(("after", after.to_string()));
        }

        if let Some(before) = value.before {
            params.push(("before", before.to_string()));
        }

        params
    }
}

#[cfg(test)]
mod tests {
    use super::Options;

    #[test]
    fn test_option_params() {
        let opts = Options::new().after("asd").count(3).limit(100).before("xd");
        let params: Vec<(&str, String)> = opts.into();

        assert_eq!(
            params,
            vec![
                ("count", "3".to_string()),
                ("limit", "100".to_string()),
                ("after", "asd".to_string()),
                ("before", "xd".to_string())
            ]
        );
    }
}
