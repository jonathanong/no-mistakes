use super::*;

#[test]
fn environment_names_are_case_insensitive_and_shadow_repository_names() {
    let findings = run("environments", ".no-mistakes.yml").unwrap();
    let locations: Vec<_> = findings
        .iter()
        .map(|finding| (finding.file.as_str(), finding.line))
        .collect();
    assert_eq!(
        locations,
        [
            (".github/workflows/check.yml", 15),
            (".no-mistakes.yml", 8),
            (".no-mistakes.yml", 14),
        ]
    );
    assert!(findings[0].message.contains("undeclared secret ENV_ONLY"));
    // The environment's SHARED consumes that declaration, leaving the repository one unused.
    assert!(findings[1]
        .message
        .contains("unused declared secret SHARED in repository"));
    assert!(findings[2]
        .message
        .contains("unused declared secret ENV_UNUSED in Prod"));
    assert!(!findings
        .iter()
        .any(|finding| finding.message.contains("REPO_ONLY")));
}

#[test]
fn unused_repository_and_environment_findings_point_at_their_config_declarations() {
    let findings = run("unused", ".no-mistakes.yml").unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [9, 12]
    );
    assert!(findings
        .iter()
        .all(|finding| finding.file == ".no-mistakes.yml"));
    assert!(findings[0].message.contains("UNUSED_REPO in repository"));
    assert!(findings[1].message.contains("UNUSED_ENV in Stage"));
}

#[test]
fn repeated_names_in_one_block_scalar_keep_each_authored_line() {
    let findings = run("repeated", ".no-mistakes.yml").unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [7, 8, 9, 10]
    );
    for finding in &findings {
        assert_eq!(finding.file, ".github/workflows/check.yml");
        assert!(
            finding.message.starts_with("undeclared secret"),
            "{finding:?}"
        );
    }
}

#[test]
fn configured_builtin_exemptions_replace_the_default_and_match_case_insensitively() {
    let findings = run("custom-builtins", ".no-mistakes.yml").unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 7);
    assert!(findings[0]
        .message
        .contains("undeclared secret GITHUB_TOKEN"));
}

#[test]
fn quoted_expression_literals_yaml_comments_and_other_contexts_do_not_create_secret_reads() {
    let findings = run("scanner", ".no-mistakes.yml").unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [7, 8, 11]
    );
    for (finding, name) in findings.iter().zip(["REAL", "BRACKET", "BARE"]) {
        assert!(
            finding
                .message
                .contains(&format!("undeclared secret {name}")),
            "{finding:?}"
        );
    }
}

#[test]
fn exempt_names_that_are_explicitly_declared_still_count_as_used() {
    assert!(run("custom-builtins", "declared.yml").unwrap().is_empty());
}

#[test]
fn job_environment_does_not_apply_to_workflow_level_env() {
    let findings = run("queued-env", ".no-mistakes.yml").unwrap();
    // ENV_ONLY stays undeclared. Its unused claim is withheld because
    // workflow-level env versus job environment is not verified.
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(findings[0].file, ".github/workflows/check.yml");
    assert_eq!(findings[0].line, 5);
    assert!(findings[0].message.contains("undeclared secret ENV_ONLY"));
    assert_eq!(findings[1].file, ".no-mistakes.yml");
    assert_eq!(findings[1].line, 12);
    assert!(findings[1]
        .message
        .contains("unused declared secret OTHER_UNUSED in stage"));
    assert!(!findings
        .iter()
        .any(|finding| finding.message.contains("unused declared secret ENV_ONLY")));
}

#[test]
fn with_input_named_if_is_not_a_bare_conditional() {
    let findings = run("with-input-if", ".no-mistakes.yml").unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(findings[0].file, ".github/workflows/check.yml");
    assert_eq!(findings[0].line, 10);
    assert!(findings[0].message.contains("undeclared secret BARE"));
    assert_eq!(findings[1].file, ".no-mistakes.yml");
    assert!(findings[1]
        .message
        .contains("unused declared secret UNUSED in repository"));
    assert!(!findings
        .iter()
        .any(|finding| finding.message.contains("incomplete")));
}

#[test]
fn double_quoted_index_is_not_a_static_secret_name() {
    let findings = run("quoted-index", ".no-mistakes.yml").unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [8, 9]
    );
    assert!(findings
        .iter()
        .all(|finding| finding.file == ".github/workflows/check.yml"));
    assert!(findings[0].message.contains("undeclared secret FOO"));
    assert!(findings[1]
        .message
        .contains("dynamic selector or whole secrets context"));
    assert!(!findings[1].message.contains("undeclared secret FOO"));
}

#[test]
fn unreferenced_action_descriptors_are_not_scanned() {
    let findings = run("unreferenced-action", ".no-mistakes.yml").unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        ["actions/inner/action.yml", "actions/outer/action.yml"]
    );
    assert!(findings.iter().all(|finding| {
        finding
            .message
            .contains("composite actions cannot access the secrets context directly")
    }));
    assert!(!findings
        .iter()
        .any(|finding| finding.file.contains("vendor") || finding.message.contains("VENDOR")));
}

#[test]
fn workflow_level_environment_references_resolve_repository_inventory() {
    let findings = run("workflow-environment", ".no-mistakes.yml").unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(findings[0].file, ".github/workflows/check.yml");
    assert_eq!(findings[0].line, 4);
    assert!(findings[0]
        .message
        .contains("undeclared secret WORKFLOW_MISSING"));
    assert_eq!(findings[1].file, ".no-mistakes.yml");
    assert_eq!(findings[1].line, 7);
    assert!(findings[1]
        .message
        .contains("unused declared secret UNUSED in repository"));
}
