pub use no_mistakes_ts_analysis::ts_resolver::*;

/// Load a `TsConfig` from `--tsconfig` if given, else search upward from `root`,
/// else return an empty config anchored at `root`.
///
/// Shared by every command that resolves import/re-export specifiers so the
/// `--tsconfig` fallback behaves identically across the CLI and N-API surface.
pub fn resolve_tsconfig(
    arg: Option<&std::path::Path>,
    root: &std::path::Path,
) -> anyhow::Result<TsConfig> {
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(root);
    resolve_tsconfig_from_visible(arg, root, &visible_paths)
}

#[path = "ts_resolver/declarations.rs"]
pub(crate) mod declarations;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod catalog_coverage_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod catalog_membership_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod catalog_reference_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "ts_resolver/session_builder_tests.rs"]
mod session_builder_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "ts_resolver/session_graph_cache_policy_tests.rs"]
mod session_graph_cache_policy_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "ts_resolver/session_graph_cache_tests.rs"]
mod session_graph_cache_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "ts_resolver/session_visible_projection_tests.rs"]
mod session_visible_projection_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "ts_resolver/session_visible_tests.rs"]
mod session_visible_tests;
#[cfg(test)]
mod tests;
