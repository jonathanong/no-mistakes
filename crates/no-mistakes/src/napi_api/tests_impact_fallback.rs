#[test]
fn tests_plan_json_binary_lockfile_fallback_matches_cli_opt_in_semantics() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/tests-plan-lockfile/binary-lockfile-fallback");
    let disabled = tests_plan_json_impl(
        json!({
            "root": root,
            "changedFiles": ["bun.lockb"],
            "globalConfigFallback": false
        })
        .to_string(),
    )
    .unwrap();
    let disabled: serde_json::Value = serde_json::from_str(&disabled).unwrap();
    assert_eq!(disabled["fallback_triggered"], false, "{disabled:?}");

    let enabled = tests_plan_json_impl(
        json!({
            "root": root,
            "changedFiles": ["bun.lockb"],
            "globalConfigFallback": true
        })
        .to_string(),
    )
    .unwrap();
    let enabled: serde_json::Value = serde_json::from_str(&enabled).unwrap();
    assert_eq!(enabled["fallback_triggered"], true, "{enabled:?}");
    assert_eq!(enabled["selected_tests"].as_array().unwrap().len(), 1);
}

#[test]
fn tests_plan_json_diff_only_fallback_matches_cli_opt_in_semantics() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/tests-plan-lockfile/diff-only-fallback");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path();
    crate::test_support::git_init(root);
    crate::test_support::git_commit_all(root, "initial fixture");
    let diff = std::fs::read_to_string(root.join("lockfile.diff")).unwrap();
    for (fallback, expected) in [(false, false), (true, true)] {
        let output = tests_plan_json_impl(
            json!({
                "root": root,
                "diff": diff,
                "base": "HEAD",
                "globalConfigFallback": fallback
            })
            .to_string(),
        )
        .unwrap();
        let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(plan["fallback_triggered"], expected, "{plan:?}");
        assert!(plan["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| { warning["type"] == "lockfile-no-baseline" }));
        assert_eq!(
            plan["selected_tests"].as_array().unwrap().len(),
            usize::from(expected),
            "{plan:?}"
        );
    }
}

include!("tests_impact_fallback/config_provenance.rs");

#[test]
fn tests_plan_json_resolves_explicit_relative_tsconfig_under_request_root() {
    let root = fixture_root("aliased");
    let options = json!({
        "root": root,
        "tsconfig": "tsconfig.json",
        "changedFiles": ["main.mts"]
    })
    .to_string();

    tests_plan_json_impl(options).unwrap();
}

#[test]
fn tests_plan_json_workspace_shared_change_selects_all_importing_projects() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/workspace-resolution"),
    );
    let output = tests_plan_json_impl(
        json!({
            "root": root,
            "framework": "vitest",
            "changedFiles": ["packages/shared/src/message.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();

    assert!(selected
        .iter()
        .any(|test| test["test_file"] == "apps/web/tests/entry.test.ts"), "{plan:#?}");
    assert!(selected
        .iter()
        .any(|test| test["test_file"] == "services/worker/tests/entry.test.ts"), "{plan:#?}");
}

#[test]
fn tests_plan_json_discovers_framework_config_aliases_from_configured_nested_runner() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/framework-project-alias"),
    );
    let output = tests_plan_json_impl(
        json!({
            "root": root,
            "framework": "vitest",
            "changedFiles": ["apps/web/src/value.ts"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(plan["selected_tests"].as_array().unwrap().iter().any(|test| {
        test["test_file"] == "apps/web/tests/value.impact.ts"
    }), "{plan:#?}");
}

// Regression for a review finding on #508: the CLI rejects --from-git-diff
// combined with --base/--head via clap's conflicts_with_all, but the N-API
// options struct isn't bound by clap. Without a matching guard in
// generate_plan, this combination would silently resolve to fromGitDiff's
// value instead of surfacing the same parity error the CLI gives.
#[test]
fn tests_plan_json_rejects_from_git_diff_with_base() {
    let root = fixture_root("tests-impact-diff");
    let options = json!({
        "root": root,
        "fromGitDiff": "origin/main...HEAD",
        "base": "origin/main"
    })
    .to_string();
    let error = tests_plan_json_impl(options).unwrap_err();
    assert!(
        error.to_string().contains("conflicts"),
        "expected a conflicts error, got: {error}"
    );
}

#[test]
fn entrypoint_option_without_symbol_parts_as_file_only() {
    assert_eq!(
        super::options::EntrypointOption::Symbol(super::options::EntrypointSymbolOption {
            file: "src/a.mts".to_string(),
            symbol: None,
        })
        .into_parts(),
        ("src/a.mts".to_string(), None)
    );
    assert_eq!(
        super::options::EntrypointOption::Symbol(super::options::EntrypointSymbolOption {
            file: "src/a.mts".to_string(),
            symbol: Some(String::new()),
        })
        .into_parts(),
        ("src/a.mts".to_string(), None)
    );
}

#[test]
fn entrypoint_option_rejects_unknown_symbol_fields() {
    let root = fixture_root("tests-impact-symbol");
    let options = json!({
        "root": root,
        "includeSymbols": true,
        "entrypoints": [{ "file": "utils.mts", "symbl": "parseDate" }]
    })
    .to_string();
    tests_plan_json_impl(options).unwrap_err();
}
