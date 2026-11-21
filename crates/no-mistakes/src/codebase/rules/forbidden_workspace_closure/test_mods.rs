// Included into the rule module so these tests stay direct children of it.
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests_catalog;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests_lockfile_config;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests_manifest_alias;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests_workspace;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests_workspace_lockfile;
