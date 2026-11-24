#[test]
fn check_reports_invalid_concurrency_policy_options() {
    let error = super::super::prepare::check_with_files(
        &super::fixture("needs-basic"),
        &super::config(
            r#"
concurrencyPolicy:
  ci.yml:
    cancellation: retain-running
    scope: [sha]
"#,
        ),
        &[],
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("missing pending"), "{error}");
}

#[test]
fn malformed_concurrency_policy_does_not_fall_back_to_the_default() {
    let parsed =
        super::config("concurrencyPolicy: []").rules[0].try_rule_options::<super::super::Options>();
    let Err(error) = parsed else {
        panic!("malformed concurrencyPolicy must fail");
    };
    let error = error.to_string();
    assert!(error.contains("concurrencyPolicy"), "{error}");
}

#[test]
fn malformed_workflow_is_a_structured_error_without_a_session() {
    let error = super::super::prepare::check_with_files(
        &super::fixture("reusable-concurrency"),
        &super::config("forbidReusableWorkflowConcurrency: true"),
        &[],
    )
    .unwrap_err();
    let failure = error
        .downcast_ref::<crate::codebase::check_outcome::CheckInputFailure>()
        .unwrap();
    assert_eq!(
        failure.file.as_deref(),
        Some(std::path::Path::new(".github/workflows/malformed.yml"))
    );
}
