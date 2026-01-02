use std::fmt;
use std::hash::Hash;
use std::sync::atomic::{self, AtomicU64};

/// The id of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Id(u64);

static COUNT: AtomicU64 = AtomicU64::new(1);

impl Id {
    /// The main window id.
    ///
    /// This is the id of the first window that is created by the application.
    /// It's typically used when you only have a single window and need to
    /// reference it without storing the id explicitly.
    pub const MAIN: Self = Self(0);

    /// Creates a new unique window [`Id`].
    pub fn unique() -> Id {
        Id(COUNT.fetch_add(1, atomic::Ordering::Relaxed))
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
