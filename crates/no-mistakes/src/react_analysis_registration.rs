use crate::codebase::ts_source::facts::react_analysis::Hooks;
use crate::{react_traits, server_routes};

pub(crate) static REACT_HOOKS: Hooks = Hooks {
    extract_components: react_traits::analyze::file::collect_components,
    extract_usages: react_traits::pipeline::usages::collect_usage_file_facts,
    extract_server_routes: server_routes::extract::extract_program,
    project_server_routes: server_routes::route_defs_from_prepared_facts_with_catalog,
};

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
