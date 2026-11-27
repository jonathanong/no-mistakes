pub(crate) use crate::codebase::playwright_facts::test_patterns::{PatternOptions, Patterns};

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod boundary_tests;
#[cfg(test)]
mod tests;
