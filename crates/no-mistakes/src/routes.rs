pub use no_mistakes_ts_analysis::routes::*;

#[cfg(test)]
use std::path::Path;

pub mod rewrites;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
