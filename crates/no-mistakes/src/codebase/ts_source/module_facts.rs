pub use no_mistakes_ts_facts::module_facts::*;

mod report;
pub use report::*;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;

#[cfg(test)]
mod selector_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod created_require_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod precision_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod static_dynamic_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "module_facts/legacy_references/tests.rs"]
mod legacy_references_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "module_facts/argument_tests/unknown_global_guards.rs"]
mod unknown_global_guards;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "module_facts/argument_line_index_tests.rs"]
mod argument_line_index_tests;
