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

    let invalid_framework =
        crate::napi_api::cli_parity::build_plan_args(crate::napi_api::options::TestsPlanOptions {
            framework: Some("jest".to_string()),
            ..Default::default()
        })
        .unwrap_err();
    assert!(invalid_framework
        .to_string()
        .contains("unknown test framework: jest"));

    let missing_test = crate::napi_api::cli_parity::build_why_args(Default::default()).unwrap_err();
    assert!(missing_test.to_string().contains("test is required"));
}
