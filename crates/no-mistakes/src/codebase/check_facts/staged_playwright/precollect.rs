use super::super::{CheckFactPlan, PlaywrightFactPlan};
use crate::codebase::ts_source::facts::TsFactMap;
use std::collections::HashSet;
use std::path::PathBuf;

pub(super) fn cached_config_graph_facts(
    files: &[PathBuf],
    graph_files: &[PathBuf],
    plan: &CheckFactPlan,
    playwright: &PlaywrightFactPlan,
) -> TsFactMap {
    if !crate::ast::request_parse_cache_active() {
        return TsFactMap::new();
    }
    let universe = files.iter().chain(graph_files).collect::<HashSet<_>>();
    let context = plan.graph_context.clone();
    TsFactMap::from_iter_with_plan(
        playwright
            .config_files()
            .iter()
            .filter(|path| universe.contains(path))
            .filter_map(|path| {
                let source = std::fs::read_to_string(path).ok()?;
                crate::ast::with_program(path, &source, |program, source| {
                    crate::codebase::ts_source::facts::collect_file_facts_from_program(
                        path, plan.graph, &context, source, program, None,
                    )
                })
                .ok()
                .map(|facts| (path.clone(), facts))
            }),
        plan.graph,
    )
}
