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
    let mut config = config.clone();
    crate::codebase::rules::analysis_policy::prepare_rule_analysis(
        root,
        &mut config,
        &visible,
        &sources,
    )
    .unwrap();
    let mut plan = CheckFactPlan::default();
    crate::codebase::postgres::assign_prepared_embedded_sql_options(&config, &mut plan).unwrap();
    crate::codebase::postgres::configure_prepared_postgres_plan(root, &config, &mut plan).unwrap();
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
    let saved = crate::test_support::materialize_saved_fixture(&fixture());
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
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
    assert_eq!(
        parses.len(),
        files
            .iter()
            .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
            .count()
    );
    assert!(parses.values().all(|count| *count == 1), "{parses:#?}");
    let (mut maps, full_count) = counts(|| vec![collect_single(&root, files.clone(), full)]);
    let full = maps.pop().unwrap();
    assert_eq!(narrow_count, 4, "caller, barrel, helper, constant");
    assert_eq!(
        full_count, 6,
        "unrelated valid and recovered malformed modules previously paid SQL extraction"
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
    let saved = crate::test_support::materialize_saved_fixture(&fixture());
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
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
    let saved = crate::test_support::materialize_saved_fixture(&fixture());
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let original = prepared(&root, &files, &config);
    let mut direct = original.clone();
    direct.query_annotation_catalog = None;
    assert!(!eligible(&direct, None));
    let (_, direct_count) = counts(|| vec![collect_check_facts(&root, files.clone(), direct)]);
    assert_eq!(direct_count, 6, "direct requests keep broad compatibility");
    let mut annotation = original.clone();
    annotation.query_annotation = true;
    assert!(!eligible(&annotation, None));
    for literal_helper in [true, false] {
        let mut unsupported = original.clone();
        let scans = unsupported
            .embedded_sql_rule_ids
            .values()
            .next()
            .unwrap()
            .clone();
        let profile = &mut unsupported.embedded_sql_options[0];
        if literal_helper {
            profile
                .sql_literal_helpers
                .push("@helpers/page#pageClause".into());
        } else {
            profile.track_transaction_helpers = true;
        }
        unsupported.embedded_sql_rule_ids.clear();
        unsupported
            .embedded_sql_rule_ids
            .insert(profile.clone(), scans);
        assert!(!eligible(&unsupported, None));
    }
    let mut head_only = original.clone();
    head_only
        .embedded_sql_head_calls_only
        .push(original.embedded_sql_options[0].clone());
    assert!(!eligible(&head_only, None));
    let mut heads = original.clone();
    heads.query_annotation_heads = true;
    assert!(!eligible(&heads, None));
    for file in ["low.yml", "broad.yml"] {
        let config = crate::config::v2::load_v2_config(&root, Some(&root.join(file))).unwrap();
        let plan = prepared(&root, &files, &config);
        if file == "low.yml" {
            assert!(!plan.sql_identifier_profiles.is_empty());
        }
        assert!(!eligible(&plan, None), "{file}");
    }
    let session = AnalysisSession::disabled();
    let sources = session.visible_paths(&root).source_store_for(&root);
    let (empty, count) = counts(|| {
        vec![collect_single(
            &root,
            vec![root.join("tsconfig.json")],
            original.clone(),
        )]
    });
    assert!(empty[0].ts.is_empty());
    assert_eq!(
        count, 0,
        "non-source-only inputs must not recurse through batch assembly"
    );
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
    assert_eq!(count, 12, "two SQL variants retain the existing broad path");
}

thread_local! {
    pub(super) static BROAD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[test]
fn actual_check_json_preparation_activates_scoping_and_preserves_full_report() {
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            BROAD.with(|value| value.set(false));
        }
    }
    let saved = crate::test_support::materialize_saved_fixture(&fixture());
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
    let run = |broad| {
        BROAD.with(|value| value.set(broad));
        let _restore = Restore;
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let report = crate::diagnostics::with_observer(Some(observer.clone()), || {
            crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                serde_json::json!({ "root": root, "config": ".no-mistakes.yml" }),
            ))
            .unwrap()
        });
        assert_eq!(
            observer.snapshot().work.get("check.fact_collections"),
            Some(&1),
            "actual full/scoped check preparation must record one collection: broad={broad}"
        );
        let count = observer
            .snapshot()
            .work
            .get("postgres.embedded.collections")
            .copied()
            .unwrap_or(0);
        (
            serde_json::from_str::<serde_json::Value>(&report).unwrap(),
            count,
        )
    };
    let (full, broad_count) = run(true);
    let (scoped, scoped_count) = run(false);
    assert_eq!(scoped, full);
    assert_eq!(broad_count, 6);
    assert_eq!(
        scoped_count, 4,
        "actual check preparation must use the scope scheduler"
    );
    assert_eq!(
        scoped["rules"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|finding| finding["rule"] == "postgres-no-offset")
            .count(),
        1
    );
}

#[test]
fn no_support_and_empty_seeds_keep_unused_workspace_uninitialized() {
    let fixture = fixture().join("../../sql-no-support/fixture");
    let saved = crate::test_support::materialize_saved_fixture(&fixture);
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    for config_name in [".no-mistakes.yml", "empty.yml"] {
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(config_name))).unwrap();
        let narrow = prepared(&root, &files, &config);
        let mut full = narrow.clone();
        full.embedded_sql = false;
        let (narrow, narrow_loads) =
            crate::codebase::workspaces::load_metrics_tests::count(&root, || {
                collect_single(&root, files.clone(), narrow)
            });
        let (full, full_loads) =
            crate::codebase::workspaces::load_metrics_tests::count(&root, || {
                collect_single(&root, files.clone(), full)
            });
        assert_eq!((narrow_loads, full_loads), (0, 0), "{config_name}");
        assert_eq!(
            findings(&root, &files, &sources, &config, &narrow),
            findings(&root, &files, &sources, &config, &full)
        );
        assert_eq!(
            findings(&root, &files, &sources, &config, &narrow).len(),
            usize::from(config_name == ".no-mistakes.yml")
        );
    }
}

#[test]
fn all_selected_payload_classifies_once_and_retains_no_seed_set() {
    let saved = crate::test_support::materialize_saved_fixture(
        &fixture().join("../../sql-no-support/fixture"),
    );
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("all-selected.yml"))).unwrap();
    let plan = prepared(&root, &files, &config);
    assert!(eligible(&plan, None));
    let source_files = files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        .collect::<Vec<_>>();
    let demands = source_files
        .iter()
        .map(|path| {
            (
                (*path).clone(),
                vec![FactDemand {
                    request: 0,
                    plan: plan.clone(),
                }],
            )
        })
        .collect();
    let session = crate::diagnostics::InvocationObserver::new(true);
    let selected = crate::diagnostics::with_observer(Some(session.clone()), || {
        seeds::select(
            &BatchCheckFactRequest {
                root,
                files: files.clone(),
                graph_files: vec![],
                plan,
                playwright: None,
                sources: crate::codebase::rules::source_store_for_files(&files),
            },
            0,
            &demands,
        )
    });
    assert!(
        selected.is_none(),
        "all-selected payloads return before retaining seeds"
    );
    assert_eq!(
        session.snapshot().work.get("sql_scope.seed_checks"),
        Some(&(source_files.len() as u64))
    );
}

#[test]
fn middle_exclusion_retains_selected_prefix_and_required_excluded_support() {
    let saved = crate::test_support::materialize_saved_fixture(&fixture());
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("middle-exclude.yml"))).unwrap();
    let plan = prepared(&root, &files, &config);
    let demands = files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        .map(|path| {
            (
                path.clone(),
                vec![FactDemand {
                    request: 0,
                    plan: plan.clone(),
                }],
            )
        })
        .collect();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let selected = crate::diagnostics::with_observer(Some(observer.clone()), || {
        seeds::select(
            &BatchCheckFactRequest {
                root: root.clone(),
                files: files.clone(),
                graph_files: vec![],
                plan: plan.clone(),
                playwright: None,
                sources: Arc::clone(&sources),
            },
            0,
            &demands,
        )
    })
    .expect("middle exclusions retain a strict subset");
    let expected = [
        "helpers/barrel.mts",
        "helpers/paging.mts",
        "src/caller.mts",
        "unrelated/dense.mts",
    ]
    .map(|path| root.join(path))
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(selected, expected);
    assert_eq!(
        observer.snapshot().work.get("sql_scope.seed_checks"),
        Some(&6)
    );
    let mut full = plan.clone();
    full.embedded_sql = false;
    let (mut maps, count) = counts(|| vec![collect_single(&root, files.clone(), plan)]);
    let narrow = maps.pop().unwrap();
    let full = collect_single(&root, files.clone(), full);
    assert_eq!(
        count, 5,
        "the excluded constant remains required supporting SQL"
    );
    assert!(!narrow.ts[&root.join("helpers/constant.mts")]
        .embedded_sql
        .is_empty());
    assert!(narrow.ts[&root.join("unrelated/broken.mts")]
        .parse_error
        .is_some());
    assert_eq!(
        narrow.ts[&root.join("unrelated/broken.mts")].parse_error,
        full.ts[&root.join("unrelated/broken.mts")].parse_error
    );
    let actual = findings(&root, &files, &sources, &config, &narrow);
    assert_eq!(actual, findings(&root, &files, &sources, &config, &full));
    assert_eq!(
        actual.len(),
        2,
        "selected caller and unrelated selected executor both retain findings"
    );
}
