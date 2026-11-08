use super::*;
use crate::codebase::check_facts::{collect_check_facts, CheckFactMap};
use crate::codebase::ts_resolver::TsConfigCatalog;
use crate::codebase::ts_source::SourceStore;
use std::path::Path;
use std::sync::Arc;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-facts/sql-support-scope/fixture"),
    )
}

fn prepared(
    root: &Path,
    _files: &[PathBuf],
    config: &crate::config::v2::NoMistakesConfig,
) -> CheckFactPlan {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible = snapshot.paths_for(root);
    let sources = snapshot.source_store_for(root);
    let mut plan = CheckFactPlan::default();
    crate::codebase::postgres::assign_prepared_embedded_sql_options(config, &mut plan).unwrap();
    crate::codebase::postgres::configure_prepared_postgres_plan(root, config, &mut plan).unwrap();
    plan.query_annotation_catalog = Some(Arc::new(TsConfigCatalog::from_visible_and_sources(
        root,
        &[root.to_path_buf()],
        &visible,
        &sources,
    )));
    plan
}

fn collect_single(root: &Path, files: Vec<PathBuf>, plan: CheckFactPlan) -> CheckFactMap {
    let session = AnalysisSession::disabled();
    let sources = session.visible_paths(root).source_store_for(root);
    crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session, root, (files, Vec::new()), plan, None, sources,
    )
}

fn counts(run: impl FnOnce() -> Vec<CheckFactMap>) -> (Vec<CheckFactMap>, u64) {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let maps = crate::diagnostics::with_observer(Some(observer.clone()), run);
    let count = observer
        .snapshot()
        .work
        .get("postgres.embedded.collections")
        .copied()
        .unwrap_or(0);
    (maps, count)
}

fn findings(
    root: &Path,
    files: &[PathBuf],
    sources: &SourceStore,
    config: &crate::config::v2::NoMistakesConfig,
    facts: &CheckFactMap,
) -> Vec<crate::codebase::rules::RuleFinding> {
    crate::codebase::rules::postgres_no_offset::check_with_files_sources_and_facts(
        root, config, files, sources, facts,
    )
    .unwrap()
}

#[test]
fn sql_seed_closure_preserves_imported_helper_findings_and_skips_unrelated_walks() {
    let root = fixture();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let narrow = prepared(&root, &files, &config);
    assert!(eligible(&narrow, None));
    let mut full = narrow.clone();
    full.embedded_sql = false;
    crate::ast::begin_parse_count(&root);
    let (mut maps, narrow_count) = counts(|| vec![collect_single(&root, files.clone(), narrow)]);
    let narrow = maps.pop().unwrap();
    let parses = crate::ast::finish_parse_count(&root);
    assert_eq!(parses.len(), files.len());
    assert!(parses.values().all(|count| *count == 1), "{parses:#?}");
    let (mut maps, full_count) = counts(|| vec![collect_single(&root, files.clone(), full)]);
    let full = maps.pop().unwrap();
    assert_eq!(narrow_count, 4, "caller, barrel, helper, constant");
    assert_eq!(
        full_count, 5,
        "unrelated valid module previously paid SQL extraction"
    );
    assert_eq!(
        findings(&root, &files, &sources, &config, &narrow),
        findings(&root, &files, &sources, &config, &full)
    );
    assert_eq!(findings(&root, &files, &sources, &config, &narrow).len(), 1);
    assert!(
        narrow.ts[&root.join("helpers/paging.mts")]
            .embedded_sql
            .len()
            == 1
    );
    assert!(narrow.ts[&root.join("unrelated/dense.mts")]
        .embedded_sql
        .is_empty());
    assert_eq!(
        narrow.ts[&root.join("unrelated/broken.mts")].parse_error,
        full.ts[&root.join("unrelated/broken.mts")].parse_error
    );
    assert_eq!(narrow.graph_files_complete, full.graph_files_complete);
}

#[test]
fn batch_mixed_graph_demand_preserves_other_variants_and_disabled_work_is_zero() {
    let root = fixture();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let sql = prepared(&root, &files, &config);
    let graph = CheckFactPlan {
        graph: crate::codebase::ts_source::facts::TsFactPlan::imports(),
        ..Default::default()
    };
    let session = AnalysisSession::disabled();
    let sources = session.visible_paths(&root).source_store_for(&root);
    let request = |plan| BatchCheckFactRequest {
        root: root.clone(),
        files: files.clone(),
        graph_files: files.clone(),
        plan,
        playwright: None,
        sources: Arc::clone(&sources),
    };
    let (maps, count) = counts(|| {
        super::super::batch::collect_check_fact_batch_with_session(
            &session,
            vec![request(sql), request(graph.clone())],
        )
    });
    assert_eq!(count, 4);
    let baseline = collect_check_facts(&root, files.clone(), graph);
    for (path, facts) in &maps[1].ts {
        assert_eq!(facts.ts.imports, baseline.ts[path].ts.imports);
        assert_eq!(facts.ts.function_calls, baseline.ts[path].ts.function_calls);
        assert!(facts.embedded_sql.is_empty());
    }
    assert_eq!(
        findings(&root, &files, &sources, &config, &maps[0]).len(),
        1
    );
    let (_, disabled) =
        counts(|| vec![collect_check_facts(&root, files, CheckFactPlan::default())]);
    assert_eq!(disabled, 0);
}

#[test]
fn direct_annotation_low_and_ambiguous_requests_keep_broad_demand() {
    let root = fixture();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let original = prepared(&root, &files, &config);
    let mut direct = original.clone();
    direct.query_annotation_catalog = None;
    assert!(!eligible(&direct, None));
    let (_, direct_count) = counts(|| vec![collect_check_facts(&root, files.clone(), direct)]);
    assert_eq!(direct_count, 5, "direct requests keep broad compatibility");
    let mut annotation = original.clone();
    annotation.query_annotation = true;
    assert!(!eligible(&annotation, None));
    let mut heads = original.clone();
    heads.query_annotation_heads = true;
    assert!(!eligible(&heads, None));
    for file in ["low.yml", "broad.yml"] {
        let config = crate::config::v2::load_v2_config(&root, Some(&root.join(file))).unwrap();
        assert!(!eligible(&prepared(&root, &files, &config), None), "{file}");
    }
    let session = AnalysisSession::disabled();
    let sources = session.visible_paths(&root).source_store_for(&root);
    let request = || BatchCheckFactRequest {
        root: root.clone(),
        files: files.clone(),
        graph_files: vec![],
        plan: original.clone(),
        playwright: None,
        sources: Arc::clone(&sources),
    };
    let (_, count) = counts(|| {
        super::super::batch::collect_check_fact_batch_with_session(
            &session,
            vec![request(), request()],
        )
    });
    assert_eq!(count, 10, "two SQL variants retain the existing broad path");
}
