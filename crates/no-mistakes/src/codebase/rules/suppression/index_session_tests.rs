use super::*;
use crate::codebase::ts_source::FileInventory;
use crate::diagnostics::InvocationObserver;

#[test]
fn session_observer_records_builds_once_across_concurrent_callers() {
    let root = fixture("fixtures/performance/suppression-index/dense");
    let path = root.join("src/worker.ts");
    let source: Arc<str> = std::fs::read_to_string(&path).unwrap().into();
    let observer = InvocationObserver::new(true);
    let index = Arc::new(DisableCommentIndex::new_observed(
        Arc::clone(&source),
        Some(Arc::clone(&observer)),
    ));
    let rule = "forbidden-calls";
    let (a, b) = rayon::join(
        || index.matching(rule, Some(15), true),
        || index.matching(rule, Some(3_000), true),
    );
    assert_eq!(a, index.matching(rule, Some(15), true));
    assert_eq!(b, index.matching(rule, Some(3_000), true));

    let work = observer.snapshot().work;
    assert_eq!(work.get("suppression.header_builds"), Some(&1));
    assert_eq!(work.get("suppression.line_index_builds"), Some(&1));
}

#[test]
fn parallel_readers_share_one_rwlock_scan_on_dense_source() {
    use rayon::prelude::*;

    let root = fixture("fixtures/performance/suppression-index/dense");
    let file = "src/worker.ts";
    let path = root.join(file);
    let observer = InvocationObserver::new(true);
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&path)));
    let sources = SourceStore::new_observed(inventory, Some(Arc::clone(&observer)));
    let source = sources.read_path(&path).unwrap();
    let base: Vec<_> = source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.starts_with("scheduler.register("))
        .map(|(index, _)| finding("forbidden-calls", file, index + 1))
        .collect();
    let results: Vec<_> = (0..8)
        .into_par_iter()
        .map(|_| {
            let mut batch = base.clone();
            suppress_rule_findings_for_path(&sources, &path, &mut batch);
            batch
        })
        .collect();
    assert!(results.iter().all(|batch| batch == &results[0]));
    assert_eq!(results[0].len(), 2_800);

    let work = observer.snapshot().work;
    assert_eq!(work.get("suppression.header_builds"), Some(&1));
    assert_eq!(work.get("suppression.line_index_builds"), Some(&1));
    let last_finding_line = base.last().unwrap().line as u64;
    assert_eq!(
        work.get("suppression.line_visits"),
        Some(&last_finding_line)
    );
    assert_eq!(work.get("suppression.standalone_fallbacks"), None);
}

#[test]
fn request_paths_use_shared_store_without_standalone_fallbacks() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "test-cases/rules/no-sparse-checkout/fixture/fail",
    ));
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let files = snapshot.paths_for(&root).as_ref().clone();
    let observer = InvocationObserver::new(true);
    let sources = Arc::new(SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(&files)),
        Some(Arc::clone(&observer)),
    ));
    let findings = crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
        crate::codebase::rules::filesystem_dispatch::run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts(
            &root,
            &config,
            &files,
            crate::codebase::rules::filesystem_dispatch::PreparedFilesystemRuleInputs {
                session: None,
                next_config_resolver: None,
                snapshot: &snapshot,
                vitest_catalog: None,
                sources: Arc::clone(&sources),
                workflow_documents: None,
                tsconfig_gate_project_inputs: None,
                config_path: None,
            },
            None,
        )
    })
    .unwrap();
    assert!(!findings.is_empty());
    let work = observer.snapshot().work;
    assert_eq!(work.get("suppression.standalone_fallbacks"), None);
    assert!(
        work.get("suppression.header_builds")
            .copied()
            .unwrap_or_default()
            >= 1
    );

    // Empty findings and missing paths are safe no-ops for the path helper.
    let mut empty = Vec::new();
    suppress_rule_findings_for_path(&sources, &root.join("missing.yml"), &mut empty);
    let mut missing = vec![finding("no-sparse-checkout", "missing.yml", 1)];
    suppress_rule_findings_for_path(&sources, &root.join("missing.yml"), &mut missing);
    assert_eq!(missing.len(), 1);

    // Direct standalone fallbacks are tracked when invoked under an observer.
    let standalone_observer = InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(Arc::clone(&standalone_observer)), || {
        let mut by_source = vec![finding("my-rule", "a.ts", 1)];
        suppress_rule_findings_with_source(&mut by_source, "kept\n");
        let mut no_store = vec![finding("no-sparse-checkout", ".github/workflows/ci.yml", 1)];
        suppress_rule_findings_inner(&root, &mut no_store, None, &[]);
    });
    assert_eq!(
        standalone_observer
            .snapshot()
            .work
            .get("suppression.standalone_fallbacks"),
        Some(&2)
    );
}
