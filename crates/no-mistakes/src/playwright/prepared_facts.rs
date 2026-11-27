mod aggregate;
mod source;
mod test_file;

use super::{analysis, graph, integration_routes, selectors};
use crate::codebase::playwright_hooks::Hooks;

pub(crate) static HOOKS: Hooks = Hooks {
    graph_settings: graph::prepare_playwright_settings,
    graph_fact_plan: graph::fact_plan,
    partial_graph_fact_plan: graph::fact_plan_partial,
    route_edges: graph::collect_playwright_route_edges_from_snapshot,
    selector_edges: graph::collect_playwright_selector_edges_with_graph,
    selector_source_files: analysis::app_collect::collect_selector_source_files_from_visible,
    source_facts: source::collect,
    static_exports: selectors::collect_static_export_values,
    test_facts: test_file::collect_playwright_facts,
    aggregate_facts: aggregate::playwright_aggregate_facts,
    runner_plan: integration_routes::runner_plan,
    route_links: integration_routes::prepare_links,
};
