use std::path::PathBuf;
use std::sync::Arc;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture")
        .join(name)
        .canonicalize()
        .unwrap()
}

fn observed_check(
    root: &std::path::Path,
) -> (serde_json::Value, crate::diagnostics::DiagnosticsSnapshot) {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let report = crate::ast::with_request_parse_cache(|| {
        let _guard = crate::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        let results = crate::check_runner::run_all_with_evidence(
            root.to_path_buf(),
            None,
            None,
            false,
            false,
        )
        .unwrap_or_else(|error| panic!("check {root:?}: {error:#}"));
        crate::check_runner::json_value(&results)
    });
    (report, observer.snapshot())
}

fn annotation_finding(report: &serde_json::Value) -> &serde_json::Value {
    let findings = report["rules"].as_array().expect("rules");
    assert_eq!(findings.len(), 1, "{report}");
    let finding = &findings[0];
    assert_eq!(finding["rule"], "postgres-require-query-annotation");
    assert!(
        finding["message"]
            .as_str()
            .is_some_and(|message| message
                .ends_with("query execution must start with a /* name */ annotation")),
        "{finding}"
    );
    finding
}

#[test]
fn omitted_import_specifier_does_not_seed_every_non_type_import() {
    let root = fixture("helper-tracing-executor-names-without-module");
    let (report, diagnostics) = observed_check(&root);
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    let finding = annotation_finding(&report);
    assert_eq!(finding["file"], "src/executor.mts");
    assert_eq!(finding["line"], 2);
    // Four parsed modules. Only the executor and the caller that reaches it
    // are roots; the unrelated value import is not a configured-module seed.
    assert_eq!(diagnostics.work["postgres.annotation.modules"], 4);
    assert_eq!(diagnostics.work["postgres.annotation.roots"], 2);
}

#[test]
fn pnpm_workspace_package_imports_resolve_and_prune_dead_ends() {
    let root = fixture("helper-tracing-pnpm-workspace");
    let (report, diagnostics) = observed_check(&root);
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    let finding = annotation_finding(&report);
    assert_eq!(finding["file"], "packages/executor/src/index.mts");
    assert_eq!(finding["line"], 2);
    // Five modules: executor, its caller, the constants package, the dead-end
    // importer, and the unresolved importer. The dead-end package import
    // resolves inside the catalog and is pruned. The missing package has no
    // evaluator target and is also pruned. A traced literal, rather than an
    // unanalyzable parameter, is what keeps the caller.
    assert_eq!(diagnostics.work["postgres.annotation.modules"], 5);
    assert_eq!(diagnostics.work["postgres.annotation.roots"], 2);
}

#[test]
fn parse_failed_inventory_target_does_not_keep_its_importer() {
    let root = fixture("helper-tracing-parse-failed-target");
    let (report, diagnostics) = observed_check(&root);
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    let finding = annotation_finding(&report);
    assert_eq!(finding["file"], "src/executor.mts");
    assert_eq!(finding["line"], 2);
    // The broken file is in the inventory but absent from the annotation
    // catalog. Its importer resolves and is not kept: a parse failure has no
    // query facts, so retaining that edge cannot yield SQL.
    assert_eq!(diagnostics.work["postgres.annotation.modules"], 2);
    assert_eq!(diagnostics.work["postgres.annotation.roots"], 1);
}

#[test]
fn excluded_mixed_executor_caller_keeps_context_but_direct_only_is_pruned() {
    let root = fixture("helper-tracing-excluded-mixed-root");
    let (report, diagnostics) = observed_check(&root);
    let finding = annotation_finding(&report);
    assert_eq!(finding["file"], "src/wrap.ts");
    assert_eq!(finding["line"], 2);
    // The mixed caller and wrapper are roots. The third, direct-only excluded
    // module must not seed demand merely because it has an executor call.
    assert_eq!(diagnostics.work["postgres.annotation.modules"], 3);
    assert_eq!(diagnostics.work["postgres.annotation.roots"], 2);
}
