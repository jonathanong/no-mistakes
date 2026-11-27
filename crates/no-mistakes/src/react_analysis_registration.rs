use crate::codebase::ts_source::facts::{
    react_analysis::{self, Hooks},
    TsFactContext,
};
use crate::{react_traits, server_routes};

static REACT_HOOKS: Hooks = Hooks {
    extract_components: react_traits::analyze::file::collect_components,
    extract_usages: react_traits::pipeline::usages::collect_usage_file_facts,
    extract_server_routes: server_routes::extract::extract_program,
    project_server_routes: server_routes::route_defs_from_prepared_facts_with_catalog,
};

// The facade wires its hooks even for callers of the direct Rust fact APIs.
impl Default for TsFactContext {
    fn default() -> Self {
        react_analysis::register(&REACT_HOOKS);
        Self::without_extensions()
    }
}
