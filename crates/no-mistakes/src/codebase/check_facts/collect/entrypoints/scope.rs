use crate::codebase::check_facts::{
    CheckFactMap, CheckFactPlan, CheckFileFacts, PlaywrightFactPlan, SupplementalFactInput,
};
use crate::codebase::ts_source::{FileIdMap, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn collect_with_scope(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>, bool),
    mut plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
    sources: Arc<SourceStore>,
    supplemental: SupplementalFactInput<'_, FileIdMap<CheckFileFacts>>,
) -> CheckFactMap {
    let SupplementalFactInput {
        facts: precollected,
        call_sites: supplemental_call_sites,
    } = supplemental;
    let (files, graph_files, graph_files_complete) = file_scope;
    let files = plan.with_production_files(files);
    if plan.graph_context.visible_files.is_none() {
        let mut visible_files = files.clone();
        visible_files.extend(graph_files.iter().cloned());
        if let Some(playwright) = &playwright {
            visible_files.extend(playwright.source_files().iter().cloned());
        }
        plan.graph_context.set_visible_files(visible_files);
    }
    if crate::codebase::check_facts::sql_scope::eligible(&plan, playwright.as_ref())
        && precollected.is_empty()
        && supplemental_call_sites.is_empty()
        && session
            .existing_sources_for(root)
            .is_some_and(|existing| Arc::ptr_eq(&existing, &sources))
        && files
            .iter()
            .any(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        let mut map = crate::codebase::check_facts::batch::collect_check_fact_batch_with_session(
            session,
            vec![crate::codebase::check_facts::batch::BatchCheckFactRequest {
                root: root.to_path_buf(),
                files,
                graph_files,
                plan,
                playwright,
                sources,
            }],
        )
        .pop()
        .expect("one requested fact map");
        map.graph_files_complete = graph_files_complete;
        return map;
    }
    session.record_work("check.fact_collections", 1);
    if let Some(playwright) = playwright {
        return crate::codebase::check_facts::staged_playwright::collect_with_sources_and_session(
            session,
            root,
            (files, graph_files, graph_files_complete),
            plan,
            playwright,
            sources,
            SupplementalFactInput {
                facts: precollected,
                call_sites: supplemental_call_sites,
            },
        );
    }
    super::super::collect_check_facts_inner(
        session,
        root,
        (files, graph_files, graph_files_complete),
        plan,
        playwright,
        sources,
        precollected,
    )
}
