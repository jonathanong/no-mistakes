#[test]
fn controlled_symbol_index_reentry_completes_with_known_dependencies() {
    const CHILD: &str = "NO_MISTAKES_SYMBOL_REENTRY_CHILD";
    const TEST: &str = concat!(
        "napi_api::analyze_project::domain_parity_tests::",
        "controlled_symbol_index_reentry_completes_with_known_dependencies"
    );
    if std::env::var_os(CHILD).is_none() {
        use wait_timeout::ChildExt;
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture"])
            .env(CHILD, "1")
            .spawn()
            .unwrap();
        eprintln!("controlled reentry child pid: {}", child.id());
        let outcome = child.wait_timeout(std::time::Duration::from_secs(5));
        // Always reap before asserting, including timeout/errors; exited children are harmless.
        let _ = child.kill();
        let _ = child.wait();
        assert!(
            matches!(outcome, Ok(Some(status)) if status.success()),
            "controlled reentry child did not complete successfully: {outcome:?}"
        );
        return;
    }

    // Scheduling control only: reuse the real saved fixture and actual reports.
    // This does not claim to reproduce the historical unmodified CI schedule.
    let root = repo_fixture(&["fixtures", "performance", "core-analysis"]);
    let options = json!({
        "root": root,
        "tsconfig": root.join("tsconfig.json"),
        "reports": [
            {"id": "symbol-dependents", "type": "dependents", "files": [{"file": "packages/core/src/index.ts", "symbol": "CoreValue"}], "relationships": ["all"]},
            {"id": "dependencies", "type": "dependencies", "files": ["src/app.tsx"], "relationships": ["all"]}
        ]
    });
    let pool = rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap();
    let (output, yields) = pool.install(|| {
        crate::codebase::dependencies::shared_symbol_index_reentry_tests::with_one_yield(|| {
            crate::ast::with_request_parse_cache(|| {
                analyze_project_json_impl(crate::napi_api::options::test_json_arg(options))
            })
            .unwrap()
        })
    });
    assert_eq!(yields, 1, "the real shared index initializer must consume the hook");
    let reports = report_results(&output);
    assert_eq!(reports.len(), 2);
    assert_eq!(
        reports[1]["files"],
        json!([
            {"path": "packages/core/src/index.ts", "depth": 1, "via": ["import"]},
            {"path": "src/button.tsx", "depth": 1, "via": ["import", "react-render"]}
        ])
    );
}
