use super::batch_report_from_prepared_facts;
use crate::codebase::queries::shared::resolve_targets;
use crate::codebase::ts_source::facts::{TsFactMap, TsFactPlan, TsFileFacts};
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn derived_resolve_check_propagates_prepared_fact_failures() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/queries/fixture");
    let target = resolve_targets(&[PathBuf::from("consumer.ts")], Some(&root), None)
        .unwrap()
        .remove(0);
    let cases = [
        (
            TsFileFacts {
                operational_error: Some("failed to read consumer.ts".to_string()),
                ..TsFileFacts::default()
            },
            "failed to read consumer.ts",
        ),
        (
            TsFileFacts {
                parse_error: Some("parser panicked".to_string()),
                fatal_parse_error: true,
                ..TsFileFacts::default()
            },
            "parser panicked",
        ),
        (
            TsFileFacts {
                fatal_parse_error: true,
                ..TsFileFacts::default()
            },
            "parser panicked without a diagnostic",
        ),
    ];
    for (file_facts, expected) in cases {
        let facts = TsFactMap::from([(target.abs_file.clone(), file_facts)]);
        let error = batch_report_from_prepared_facts(
            &target.root,
            [target.abs_file.clone()],
            &facts,
            target.visible_files(),
            &target.sources,
            None,
            &target.session,
        )
        .err()
        .expect("prepared fact failure must abort the report");
        assert!(error.to_string().contains(expected), "{error:#}");
    }
}

#[test]
fn prepared_batch_reuses_one_resolver_cache_for_one_tsconfig() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/queries/fixture");
    let targets = resolve_targets(
        &[
            PathBuf::from("consumer.ts"),
            PathBuf::from("consumer.test.ts"),
        ],
        Some(&root),
        None,
    )
    .unwrap();
    let files = targets
        .iter()
        .map(|target| target.abs_file.clone())
        .collect::<Vec<_>>();
    let facts = crate::codebase::ts_source::facts::collect_ts_facts(
        &files,
        crate::codebase::ts_source::facts::TsFactPlan::imports(),
    );
    batch_report_from_prepared_facts(
        &root,
        files,
        &facts,
        targets[0].visible_files(),
        &targets[0].sources,
        None,
        &targets[0].session,
    )
    .unwrap();

    assert_eq!(
        targets[0].session.resolver_cache_request_count_for_test(),
        1,
        "one effective tsconfig must request one session resolver cache"
    );
}

#[test]
fn prepared_batch_keeps_distinct_nearest_visible_tsconfig_scopes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/workspace-resolution")
        .canonicalize()
        .unwrap();
    let files = vec![
        root.join("apps/web/src/entry.ts"),
        root.join("apps/web/tests/entry.test.ts"),
        root.join("packages/base-owner/src/value.ts"),
    ];
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(&root);
    let visible = visible_paths
        .iter()
        .cloned()
        .collect::<crate::fx::PathSet>();
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&visible_paths)));
    let facts = TsFactMap::from_iter_with_plan(
        files
            .iter()
            .cloned()
            .map(|file| (file, TsFileFacts::default())),
        TsFactPlan::imports(),
    );
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();

    let report =
        batch_report_from_prepared_facts(&root, files, &facts, &visible, &sources, None, &session)
            .unwrap();

    assert_eq!(report.results.len(), 3);
    assert_eq!(
        session.resolver_cache_request_count_for_test(),
        2,
        "the two web files share their nearest visible config while the package file uses its own"
    );
}
