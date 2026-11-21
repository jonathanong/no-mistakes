#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "callback_tests/tests/branch_join.rs"]
mod branch_join;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "callback_tests/tests/computed_receiver.rs"]
mod computed_receiver;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "callback_tests/tests/siblings.rs"]
mod siblings;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "callback_tests/tests/stateful_revisit.rs"]
mod stateful_revisit;
