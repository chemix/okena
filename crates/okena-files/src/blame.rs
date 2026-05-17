//! Per-line blame data + provider trait, kept independent of any git
//! implementation so `okena-files` doesn't depend on `okena-git`. Concrete
//! providers (local gix, remote API) live in `okena-views-git`.

use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BlameKind {
    Committed,
    Uncommitted,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BlameCommit {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub author_email: String,
    pub timestamp: i64,
    pub summary: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BlameLine {
    pub line_number: usize,
    pub commit: Arc<BlameCommit>,
    pub kind: BlameKind,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BlameError {
    NotGitRepo,
    NotTracked,
    NoCommits,
    Backend(String),
}

impl std::fmt::Display for BlameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotGitRepo => f.write_str("not a git repository"),
            Self::NotTracked => f.write_str("file is not tracked"),
            Self::NoCommits => f.write_str("repository has no commits"),
            Self::Backend(msg) => write!(f, "blame failed: {msg}"),
        }
    }
}

/// Source of per-file blame data. Implemented by a local gix-backed provider
/// or a remote-API provider in `okena-views-git`.
///
/// Called from a background thread (`smol::unblock`), so implementations may
/// block on I/O.
pub trait BlameProvider: Send + Sync + 'static {
    fn get_blame(&self, relative_path: &str) -> Result<Vec<BlameLine>, BlameError>;
}
