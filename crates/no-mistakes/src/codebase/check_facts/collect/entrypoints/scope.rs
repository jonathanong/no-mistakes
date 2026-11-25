use crate::codebase::check_facts::{
    CheckFactMap, CheckFactPlan, CheckFileFacts, PlaywrightFactPlan, SupplementalFactInput,
};
use crate::codebase::ts_source::{FileIdMap, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[path = "vitest_spreads.rs"]
mod vitest_spreads;

pub(super) fn collect_with_scope(
    session: crate::codebase::check_facts::FactRequest<'_>,
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
    let sql_seeds = if crate::codebase::check_facts::sql_scope::eligible(&plan, playwright.as_ref())
        && precollected.is_empty()
        && supplemental_call_sites.is_empty()
        && session
            .existing_sources_for(root)
            .is_some_and(|existing| Arc::ptr_eq(&existing, &sources))
        && files
            .iter()
            .any(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        crate::codebase::check_facts::sql_scope::seeds::single(root, &files, &plan)
    } else {
        None
    };
    if let Some(sql_seeds) = sql_seeds {
        let mut map = crate::codebase::check_facts::batch::collect_check_fact_batch_with_sql_seeds(
            session.session,
            vec![crate::codebase::check_facts::batch::BatchCheckFactRequest {
                integration_discovery: session.discovery.clone(),
                root: root.to_path_buf(),
                files,
                graph_files,
                plan,
                playwright,
                sources: Arc::clone(&sources),
            }],
            Some(sql_seeds),
        )
        .pop()
        .expect("one requested fact map");
        map.graph_files_complete = graph_files_complete;
        return map;
    }
    session.record_work("check.fact_collections", 1);
    let mut supplemental_plan = CheckFactPlan {
        vitest_default_exclude_paths: Arc::clone(&plan.vitest_default_exclude_paths),
        test_reliability_paths: Arc::clone(&plan.test_reliability_paths),
        ..Default::default()
    };
    supplemental_plan.require_call_site_paths(supplemental_call_sites);
    let mut map = if let Some(playwright) = playwright {
        crate::codebase::check_facts::staged_playwright::collect_with_sources_and_session(
            session,
            root,
            (files, graph_files, graph_files_complete),
            plan,
            playwright,
            Arc::clone(&sources),
            SupplementalFactInput {
                facts: precollected,
                call_sites: supplemental_call_sites,
            },
        )
    } else {
        super::super::collect_check_facts_inner(
            session,
            root,
            (files, graph_files, graph_files_complete),
            plan,
            playwright,
            Arc::clone(&sources),
            precollected,
        )
    };
    collect_missing(
        session,
        root,
        &sources,
        supplemental_call_sites,
        &supplemental_plan,
        &mut map,
    );
    vitest_spreads::prepare(session, root, &sources, &supplemental_plan, &mut map);
    map
}

fn collect_missing(
    session: crate::codebase::check_facts::FactRequest<'_>,
    root: &Path,
    sources: &SourceStore,
    supplemental_call_sites: &[PathBuf],
    plan: &CheckFactPlan,
    facts: &mut CheckFactMap,
) {
    // Keep the requested spelling as the address of a supplemental read failure.
    let missing: std::collections::BTreeSet<_> = supplemental_call_sites
        .iter()
        .chain(plan.vitest_default_exclude_paths.iter())
        .chain(plan.test_reliability_paths.keys())
        .filter(|path| !facts.ts.contains_key(path))
        .cloned()
        .collect();
    if missing.is_empty() {
        return;
    }
    let supplemental = super::super::collect_fact_map_with_sources(
        session,
        root,
        &missing.into_iter().collect::<Vec<_>>(),
        plan,
        None,
        sources,
    );
    facts.stats.files_discovered += supplemental.len();
    facts.stats.files_parsed += supplemental.values().filter(|file| file.parsed).count();
    facts.stats.parse_errors += supplemental
        .values()
        .filter(|file| file.parse_error.is_some())
        .count();
    facts.ts.extend(supplemental.map_values(Arc::new));
}
