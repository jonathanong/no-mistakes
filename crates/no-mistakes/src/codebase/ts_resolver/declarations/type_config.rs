pub use no_mistakes_ts_analysis::ts_resolver::declarations_type_config::*;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "type_config_tests.rs"]
mod type_config_tests;
