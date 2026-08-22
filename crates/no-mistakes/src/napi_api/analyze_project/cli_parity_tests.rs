use std::path::PathBuf;

#[test]
fn cli_parity_builders_cover_defaults_and_validation() {
    let plan =
        crate::napi_api::cli_parity::build_plan_args(crate::napi_api::options::TestsPlanOptions {
            framework: Some("vitest".to_string()),
            root: Some("project".to_string()),
            changed_files: vec!["src/app.ts".to_string()],
            entrypoints: vec![
                crate::napi_api::options::EntrypointOption::Path("src/app.ts".to_string()),
                crate::napi_api::options::EntrypointOption::Symbol(
                    crate::napi_api::options::EntrypointSymbolOption {
                        file: "src/api.ts".to_string(),
                        symbol: Some("handler".to_string()),
                    },
                ),
            ],
            include_symbols: true,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(plan.environment, "pre-push");
    assert_eq!(plan.entrypoints, vec!["src/app.ts", "src/api.ts"]);
    assert_eq!(
        plan.entrypoint_symbols,
        vec![None, Some("handler".to_string())]
    );

    // from_git_diff passes through to PlanArgs unchanged, alongside base/head
    // staying unset — the CLI's --base/--head desugar happens later in
    // `collect_changed_files`, not in this options-to-args mapping.
    let from_git_diff_plan =
        crate::napi_api::cli_parity::build_plan_args(crate::napi_api::options::TestsPlanOptions {
            framework: Some("vitest".to_string()),
            from_git_diff: Some("origin/main...HEAD".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        from_git_diff_plan.from_git_diff,
        Some("origin/main...HEAD".to_string())
    );
    assert_eq!(from_git_diff_plan.base, None);
    assert_eq!(from_git_diff_plan.head, None);

    let why =
        crate::napi_api::cli_parity::build_why_args(crate::napi_api::options::TestsWhyOptions {
            test: Some("src/app.test.ts".to_string()),
            changed: Some("src/app.ts".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(why.test, PathBuf::from("src/app.test.ts"));
    assert_eq!(why.changed, Some(PathBuf::from("src/app.ts")));

    let impacted = crate::napi_api::cli_parity::build_impacted_checks_args(
        crate::napi_api::options::ImpactedChecksOptions {
            changed_files: vec!["src/app.ts".to_string()],
            diff: Some("diff --git a/src/app.ts b/src/app.ts".to_string()),
            ..Default::default()
        },
    );
    assert_eq!(impacted.changed_file, vec![PathBuf::from("src/app.ts")]);
    assert!(impacted.diff.is_none());
    assert_eq!(
        impacted.diff_content,
        Some("diff --git a/src/app.ts b/src/app.ts".to_string())
    );

    let stdin_impacted = crate::napi_api::cli_parity::build_impacted_checks_args(
        crate::napi_api::options::ImpactedChecksOptions {
            diff_stdin: true,
            diff_command: Some("git diff".to_string()),
            ..Default::default()
        },
    );
    assert!(stdin_impacted.diff_stdin);
    assert_eq!(stdin_impacted.diff_command.as_deref(), Some("git diff"));

    let impact = crate::napi_api::cli_parity::build_impact_args(
        crate::napi_api::options::TestsImpactOptions {
            root: Some("project".to_string()),
            entrypoints: vec![crate::napi_api::options::EntrypointOption::Symbol(
                crate::napi_api::options::EntrypointSymbolOption {
                    file: "src/api.ts".to_string(),
                    symbol: Some("handler".to_string()),
                },
            )],
            include_symbols: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(impact.entrypoints, vec!["src/api.ts"]);
    assert_eq!(impact.entrypoint_symbols, vec![Some("handler".to_string())]);
    assert!(impact.include_symbols);

    let jest =
        crate::napi_api::cli_parity::build_plan_args(crate::napi_api::options::TestsPlanOptions {
            framework: Some("jest".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(jest.framework, Some(crate::tests::TestFramework::Jest));

    let invalid_framework =
        crate::napi_api::cli_parity::build_plan_args(crate::napi_api::options::TestsPlanOptions {
            framework: Some("mocha".to_string()),
            ..Default::default()
        })
        .unwrap_err();
    assert!(invalid_framework
        .to_string()
        .contains("unknown test framework: mocha"));

    let missing_test = crate::napi_api::cli_parity::build_why_args(Default::default()).unwrap_err();
    assert!(missing_test.to_string().contains("test is required"));
}

#[test]
fn cli_parity_document_wrappers_accept_inline_plan_values() {
    let plan = serde_json::json!({
        "selected_tests": [
            {
                "test_file": "tests/app.test.ts",
                "confidence": "high",
                "reasons": [
                    {
                        "changed_file": "src/app.ts",
                        "path": ["src/app.ts", "tests/app.test.ts"],
                        "via": ["Test"]
                    }
                ]
            }
        ],
        "warnings": [],
        "fallback_triggered": false,
        "fallback_reason": null
    });
    let options = serde_json::json!({ "planJson": plan }).to_string();

    let comment = crate::napi_api::cli_parity::tests_comment_markdown_impl(
        crate::napi_api::options::test_json_arg(options.clone()),
    )
        .expect("inline plan should render markdown");
    assert!(comment.contains("tests/app.test.ts"));

    let graph = crate::napi_api::cli_parity::tests_graph_json_impl(crate::napi_api::options::test_json_arg(options.clone()))
        .expect("inline plan should render graph JSON");
    assert!(graph.contains("\"nodes\""));
    assert!(graph.contains("src/app.ts"));

    let mermaid = crate::napi_api::cli_parity::tests_graph_mermaid_impl(
        crate::napi_api::options::test_json_arg(options),
    )
        .expect("inline plan should render Mermaid");
    assert!(mermaid.contains("graph TD"));
}

#[test]
fn cli_parity_document_wrappers_cover_string_plan_and_required_input() {
    let raw_plan = serde_json::json!({
        "selected_tests": [],
        "warnings": [],
        "fallback_triggered": true,
        "fallback_reason": "coverage validation"
    })
    .to_string();
    let options = serde_json::json!({ "planJson": raw_plan }).to_string();

    let comment = crate::napi_api::cli_parity::tests_comment_markdown_impl(
        crate::napi_api::options::test_json_arg(options),
    )
        .expect("string plan JSON should render markdown");
    assert!(comment.contains("Fallback Triggered"));

    let error = crate::napi_api::cli_parity::tests_graph_json_impl(crate::napi_api::options::test_json_arg(serde_json::json!({})))
        .expect_err("plan input is required");
    assert!(error.reason.contains("plan or planJson is required"));

    let error = crate::napi_api::cli_parity::tests_graph_mermaid_impl(
        crate::napi_api::options::test_json_arg("{}".to_string()),
    )
        .expect_err("plan input is required");
    assert!(error.reason.contains("plan or planJson is required"));
}
