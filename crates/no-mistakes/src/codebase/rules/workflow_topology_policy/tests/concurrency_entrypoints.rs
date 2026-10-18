use super::*;

#[test]
fn check_reports_invalid_concurrency_policy_options() {
    let error = check_with_files(
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
fn empty_concurrency_policy_keeps_existing_findings_unchanged() {
    let findings = super::run(&super::fixture("needs-basic"), "concurrencyPolicy: {}\n");
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("concurrency")),
        "{findings:?}"
    );
    let baseline = super::run(
        &super::fixture("needs-basic"),
        r#"
jobInventory:
  .github/workflows/pipeline.yml: [build, test, deploy]
"#,
    );
    assert!(baseline.is_empty(), "{baseline:?}");
}
