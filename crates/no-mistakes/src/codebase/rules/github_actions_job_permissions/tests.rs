use super::{check, check_with_files_and_sources, diagnostics_with_files_and_sources, RULE_ID};
use crate::codebase::rules::{source_store_for_files, RuleFinding};
use crate::codebase::ts_source::discover_files;
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn fixture(scenario: &str) -> (PathBuf, NoMistakesConfig, Vec<PathBuf>) {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-job-permissions/fixture")
            .join(scenario),
    );
    let config =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let files = discover_files(&root, &[]);
    (root, config, files)
}

fn raw(scenario: &str) -> Vec<RuleFinding> {
    let (root, config, files) = fixture(scenario);
    let sources = source_store_for_files(&files);
    check_with_files_and_sources(&root, &config, &files, &sources).unwrap()
}

fn targets(findings: &[RuleFinding], file: &str) -> Vec<String> {
    let mut result: Vec<_> = findings
        .iter()
        .filter(|finding| finding.file.ends_with(file))
        .map(|finding| finding.target.clone().unwrap_or_default())
        .collect();
    result.sort();
    result
}

#[test]
fn inherited_writes_reach_regular_and_reusable_jobs() {
    let findings = raw("fail");
    assert_eq!(findings.len(), 4, "{findings:?}");
    assert_eq!(targets(&findings, "inherited.yml"), ["build", "call"]);
    assert_eq!(targets(&findings, "write-all.yaml"), ["build", "call"]);
    for finding in &findings {
        assert_eq!(finding.rule, RULE_ID);
        assert!(finding.line > 0);
        assert!(finding.message.contains("permissions"), "{finding:?}");
    }
    assert!(findings
        .iter()
        .any(|finding| finding.file.ends_with("inherited.yml")
            && finding.target.as_deref() == Some("build")
            && finding.line == 8));
    assert!(findings
        .iter()
        .any(|finding| finding.file.ends_with("inherited.yml")
            && finding.message.contains("id-token")));
}

#[test]
fn explicit_job_permissions_and_read_only_or_unspecified_roots_are_clean() {
    let (root, config, _) = fixture("pass");
    assert!(check(&root, &config).unwrap().is_empty());
    assert!(raw("pass").is_empty());
}

#[test]
fn uncertain_permissions_follow_unanalyzable_policy() {
    let (root, mut config, files) = fixture("unresolved");
    let sources = source_store_for_files(&files);
    let reported = diagnostics_with_files_and_sources(&root, &config, &files, &sources, true)
        .unwrap()
        .findings;
    for (file, target) in [
        ("expression.yml", "unanalyzable"),
        ("types.yml", "null-permissions"),
        ("types.yml", "list-permissions"),
        ("types.yml", "expression-permissions"),
        ("types.yml", "invalid-access"),
        ("types.yml", "unknown-scope"),
        ("malformed.yml", "unanalyzable"),
        ("jobs-shape.yml", "unanalyzable"),
        ("root-null.yml", "unanalyzable"),
        ("root-invalid-access.yml", "unanalyzable"),
        ("root-unknown-scope.yml", "unanalyzable"),
        ("root-list.yml", "unanalyzable"),
    ] {
        assert!(
            reported
                .iter()
                .any(|finding| finding.file.ends_with(file)
                    && finding.target.as_deref() == Some(target)),
            "missing {file}/{target}: {reported:?}"
        );
    }
    for (name, expected) in [("advise", reported.len()), ("ignore", 0)] {
        config.rules[0].options = serde_yaml::from_str(&format!("unanalyzable: {name}")).unwrap();
        let output =
            diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
        assert!(output.findings.is_empty(), "{:?}", output.findings);
        assert_eq!(output.advisories.len(), expected);
    }
}

#[test]
fn unknown_scope_policy_keeps_independent_known_write() {
    let (root, mut config, files) = fixture("mixed");
    let sources = source_store_for_files(&files);
    for (policy, unresolved_findings, unresolved_advisories) in
        [("report", 1, 0), ("advise", 0, 1), ("ignore", 0, 0)]
    {
        config.rules[0].options = serde_yaml::from_str(&format!("unanalyzable: {policy}")).unwrap();
        let output =
            diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
        assert_eq!(
            targets(&output.findings, "mixed.yml"),
            if unresolved_findings == 1 {
                vec!["build", "unanalyzable"]
            } else {
                vec!["build"]
            }
        );
        assert_eq!(output.advisories.len(), unresolved_advisories);
        if unresolved_advisories == 1 {
            assert_eq!(output.advisories[0].target.as_deref(), Some("unanalyzable"));
        }
    }
}

#[test]
fn next_line_line_and_file_directives_hide_inherited_writes() {
    let (root, config, files) = fixture("suppression");
    let sources = source_store_for_files(&files);
    let unsuppressed = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
    assert_eq!(unsuppressed.len(), 4, "{unsuppressed:?}");
    let visible = check(&root, &config).unwrap();
    assert_eq!(targets(&visible, "directives.yml"), ["visible"]);
    assert_eq!(visible.len(), 1, "{visible:?}");
}

#[test]
fn include_and_exclude_limit_workflow_catalog() {
    let (root, config, files) = fixture("scope");
    let sources = source_store_for_files(&files);
    let findings = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("selected"));
    assert!(findings[0].file.ends_with("selected.yml"));
}

#[test]
fn unknown_rule_options_fail_configuration() {
    let (root, mut config, files) = fixture("pass");
    let sources = source_store_for_files(&files);
    config.rules[0].options = serde_yaml::from_str("unrecognized: true").unwrap();
    let error = diagnostics_with_files_and_sources(&root, &config, &files, &sources, true)
        .err()
        .expect("unknown options should fail");
    assert!(error.to_string().contains("invalid options"), "{error}");
}

#[test]
fn flow_quoted_keys_aliases_and_merges_do_not_hide_inherited_writes() {
    // A merge without its own permissions is uncertain, but cannot hide a known root write.
    let (root, config, files) = fixture("flow");
    let sources = source_store_for_files(&files);
    let output =
        diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
    let all: Vec<_> = output
        .findings
        .iter()
        .chain(output.advisories.iter())
        .collect();
    for (file, target) in [
        ("flow.yml", "quoted-job"),
        ("alias.yml", "inherited"),
        ("merge.yml", "merged"),
    ] {
        assert!(
            all.iter()
                .any(|finding| finding.file.ends_with(file)
                    && finding.target.as_deref() == Some(target)),
            "missing {file}/{target}: {all:?}"
        );
    }
    assert!(!all
        .iter()
        .any(|finding| finding.target.as_deref() == Some("explicit")));
}
