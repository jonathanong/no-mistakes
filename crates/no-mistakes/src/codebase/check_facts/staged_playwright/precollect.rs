use super::super::{CheckFactPlan, PlaywrightFactPlan};
use crate::codebase::ts_source::facts::TsFactMap;
use std::collections::HashSet;
use std::path::PathBuf;

pub(super) fn cached_config_graph_facts(
    session: &crate::codebase::analysis_session::AnalysisSession,
    files: &[PathBuf],
    graph_files: &[PathBuf],
    plan: &CheckFactPlan,
    playwright: &PlaywrightFactPlan,
    sources: &crate::codebase::ts_source::SourceStore,
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
                let source = sources.read_path(path).ok()?;
                let mut facts = session
                    .with_program(path, &source, |program, source| {
                        crate::codebase::ts_source::facts::collect_file_facts_from_program(
                            path, plan.graph, &context, source, program, None,
                        )
                    })
                    .ok()?;
                if plan.graph.source {
                    facts.source = Some(source.to_string());
                }
                Some((path.clone(), facts))
            }),
        plan.graph,
    )
}
