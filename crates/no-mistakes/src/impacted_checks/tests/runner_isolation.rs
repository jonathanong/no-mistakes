use super::*;

fn fixture() -> tempfile::TempDir {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/runner-isolation"),
    );
    crate::test_support::materialize_saved_fixture(&source)
}

#[test]
fn impacted_checks_reuse_one_parse_pass_without_cross_runner_tests() {
    let fixture = fixture();
    let root = fixture.path().canonicalize().unwrap();
    let args = ImpactedChecksArgs {
        files: vec![PathBuf::from("tests/unit.test.mts")],
        root: root.clone(),
        config: None,
        tsconfig: None,
        base: None,
        head: None,
        changed_file: Vec::new(),
        changed_files: None,
        diff: None,
        diff_stdin: false,
        diff_command: None,
        diff_content: None,
        format: None,
        json: false,
        generic_only: false,
        timings: false,
        diagnose_empty: false,
    };

    crate::ast::begin_parse_count(&root);
    let (report, stats) = generate_impacted_checks_with_stats(&args).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let test_commands = report
        .checks
        .iter()
        .filter(|check| check.kind == CheckKind::Test)
        .map(|check| check.command.join(" "))
        .collect::<Vec<_>>();

    assert_eq!(
        test_commands.len(),
        1,
        "unrelated Playwright, .NET, and Swift suites must stay out: {test_commands:#?}"
    );
    assert_eq!(
        test_commands,
        ["vitest --config vitest.config.ts --project unit tests/unit.test.mts"],
        "the Vitest-owned file must retain its configured project target",
    );
    assert_eq!(stats.framework_discoveries, 4);
    assert_eq!(stats.graph_builds, 1);
    assert_eq!(counts.get(&root.join("vitest.config.ts")), Some(&1));
    assert_eq!(counts.get(&root.join("playwright.config.ts")), Some(&1));
    assert!(
        counts.values().all(|count| *count == 1),
        "impacted-check fanout must reuse the request's AST facts: {counts:#?}"
    );
}
