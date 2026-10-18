use super::*;

#[test]
fn dynamic_selectors_whole_context_and_dynamic_environments_are_explicitly_incomplete() {
    let findings = run("dynamic", ".no-mistakes.yml").unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.line, finding.message.contains("undeclared")))
            .collect::<Vec<_>>(),
        [(6, false), (7, false), (12, false), (15, true)]
    );
    assert!(findings
        .iter()
        .all(|finding| finding.file == ".github/workflows/check.yml"));
    assert!(findings[0]
        .message
        .contains("dynamic selector or whole secrets context"));
    assert!(findings[1]
        .message
        .contains("dynamic selector or whole secrets context"));
    assert!(findings[2]
        .message
        .contains("environment is not a literal name"));
    assert!(findings[3].message.contains("undeclared secret NOT_LISTED"));
    assert!(!findings.iter().any(|finding| {
        finding.message.contains("ENV_ONLY")
            || finding.message.contains("GITHUB_TOKEN")
            || finding.message.contains("EXTRA")
            || finding.message.contains("KNOWN")
            || finding.message.starts_with("unused declared")
    }));
}

#[test]
fn composite_direct_secrets_are_invalid_even_when_declared_or_passed_by_the_caller() {
    let findings = run("composite", ".no-mistakes.yml").unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [7, 9]
    );
    assert!(findings
        .iter()
        .all(|finding| finding.file == "actions/example/action.yml"));
    assert!(findings.iter().all(|finding| {
        finding
            .message
            .contains("composite actions cannot access the secrets context directly")
    }));
    assert!(!findings
        .iter()
        .any(|finding| finding.message.starts_with("unused declared")));
}

#[test]
fn malformed_workflow_and_action_documents_prevent_unused_findings() {
    let findings = run("malformed", ".no-mistakes.yml").unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        [".github/workflows/check.yml", "actions/broken/action.yml",]
    );
    assert!(findings
        .iter()
        .all(|finding| finding.line == 1 && finding.message.contains("document unavailable:")));
}

#[test]
fn malformed_jobs_container_is_incomplete_instead_of_proving_every_secret_unused() {
    let findings = run("malformed-jobs", ".no-mistakes.yml").unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, ".github/workflows/check.yml");
    assert_eq!(findings[0].line, 2);
    assert!(findings[0]
        .message
        .starts_with("secret inventory analysis incomplete:"));
    assert!(findings[0].message.contains("jobs must be a mapping"));
}

#[test]
fn scalar_job_value_cannot_prove_unused_inventory() {
    let findings = run("malformed-job", ".no-mistakes.yml").unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, ".github/workflows/check.yml");
    assert_eq!(findings[0].line, 3);
    assert!(findings[0].message.contains("job must be a mapping"));
}
