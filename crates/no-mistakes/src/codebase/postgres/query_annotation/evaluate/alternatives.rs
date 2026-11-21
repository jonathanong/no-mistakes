//! Alternative arms restore input state and join effects conservatively.
//! Imported modules keep one request-local initialization identity.
mod arena;
mod bindings;
mod callback_seen;
mod extras;
mod final_prune;
pub(super) mod frames;
mod freshness;
mod merge;
pub(super) mod modules;
mod prune;
mod reachable;
mod readonly;
mod run;
mod updates;
mod values;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod capture_restoration_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod deletion_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod prefix_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod shared_module_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod callback_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod dense_fixture_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod observer_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod reference_binding_tests;

#[cfg(test)]
pub(super) mod value_equality_test_support;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "alternatives/run/capture_prefix_tests.rs"]
mod capture_prefix_tests;

mod prefix_compare;
