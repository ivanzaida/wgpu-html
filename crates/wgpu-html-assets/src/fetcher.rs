use std::time::Duration;

use crate::FetchConfig;

pub struct FetchResponse {
    pub bytes: Vec<u8>,
    pub max_age: Option<Duration>,
}

/// A fetcher resolves a URL to bytes. Implementations are **synchronous**
/// and blocking — the async dispatch (thread pool, polling) lives in
/// `AssetIo`, not here.
pub trait Fetcher: Send + Sync {
    fn fetch(&self, url: &str, config: &FetchConfig) -> Option<FetchResponse>;
}
