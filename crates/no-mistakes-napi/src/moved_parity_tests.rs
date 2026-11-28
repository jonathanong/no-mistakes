use no_mistakes::impacted_checks::{generate_impacted_checks, ImpactedChecksArgs};
use no_mistakes::server_routes::analyze_project;
use std::path::{Path, PathBuf};

fn lang_fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/lang-frontends")
            .join(name),
    )
}

#[test]
fn impacted_checks_napi_matches_the_cli_engine_for_multi_framework_fanout() {
    let fixture = no_mistakes::test_support::materialize_gitignore_fixture("test-plan-fanout");
    let args = ImpactedChecksArgs {
        files: vec![PathBuf::from("src/shared.ts")],
        root: fixture.path().to_path_buf(),
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
    let cli_report = generate_impacted_checks(&args).unwrap();
    let options = serde_json::json!({
        "root": fixture.path().display().to_string(),
        "changedFiles": ["src/shared.ts"],
    });

    let napi_json =
        crate::impacted_checks_json_impl(crate::options::test_json_arg(options)).unwrap();
    let napi_report: serde_json::Value = serde_json::from_str(&napi_json).unwrap();

    assert_eq!(napi_report, serde_json::to_value(cli_report).unwrap());
}

#[test]
fn laravel_and_phoenix_route_json_matches_async_napi() {
    assert_route_json_parity(&lang_fixture("php-laravel"), "/hidden");
    assert_route_json_parity(&lang_fixture("phoenix-routes"), "/accounts/new");
    let root = lang_fixture("php-laravel");
    let options = serde_json::json!({
        "root": root.display().to_string(),
        "files": ["app/Http/Controllers/PhotoCommentController.php"],
        "relationships": ["route"]
    });
    let output = crate::dependents_json_impl(crate::options::test_json_arg(options)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(value["files"].as_array().unwrap().iter().any(|file| {
        file["path"]
            .as_str()
            .is_some_and(|path| path.ends_with("routes/web.php"))
    }));
}

fn assert_route_json_parity(root: &Path, needle: &str) {
    let report = analyze_project(root, None, &[]).unwrap();
    let cli: serde_json::Value =
        serde_json::from_str(&no_mistakes::cli::json_string(&report.routes)).unwrap();
    let options = crate::options::test_json_arg(serde_json::json!({
        "root": root.display().to_string()
    }));
    let list: serde_json::Value =
        serde_json::from_str(&crate::server_route_list_json_impl(options.clone()).unwrap())
            .unwrap();
    let full: serde_json::Value =
        serde_json::from_str(&crate::server_routes_json_impl(options).unwrap()).unwrap();
    assert_eq!(list, cli);
    assert_eq!(full["routes"], cli);
    assert!(list
        .as_array()
        .unwrap()
        .iter()
        .any(|route| route["route"] == needle));
}
