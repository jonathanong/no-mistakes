#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod bounds_functions_catalog_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod ddl_compat_schema_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod grouping_rebase_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod grouping_recovery_native_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod location_demand_embedded_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod mapped_demand_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod migration_recovery_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod select_mutation_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod select_recursive_forward_tests;
