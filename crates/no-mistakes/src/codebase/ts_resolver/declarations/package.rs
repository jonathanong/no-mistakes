pub use no_mistakes_ts_analysis::ts_resolver::declarations_package::*;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "package_tests.rs"]
mod tests;
#[cfg(test)]
use std::path::{Path, PathBuf};
