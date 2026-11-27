pub(crate) use no_mistakes_ts_facts::builtin_receivers::*;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "builtin_receivers_tests.rs"]
mod tests;
