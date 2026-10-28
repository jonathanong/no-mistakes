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
        ("missing-jobs.yml", "unanalyzable"),
        ("empty-jobs.yml", "unanalyzable"),
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
    // A merge without its own permissions is uncertain and must be reported.
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

#[test]
fn merge_and_nonmapping_shapes_report_uncertainty_without_hiding_explicit_jobs() {
    let findings = raw("edge");
    for (file, target) in [
        ("job-merge.yml", "merged"),
        ("job-merge.yml", "explicit"),
        ("jobs-merge.yml", "unanalyzable"),
        ("jobs-merge.yml", "explicit"),
        ("root-list.yml", "unanalyzable"),
        ("nonstring-job.yml", "unanalyzable"),
        ("nonstring-job.yml", "explicit"),
        ("root-merge.yml", "unanalyzable"),
        ("root-merge.yml", "explicit"),
        ("permissions-merge.yml", "unanalyzable"),
        ("permissions-merge.yml", "build"),
        ("nonstring-scope.yml", "unanalyzable"),
        ("nonstring-scope.yml", "build"),
    ] {
        assert!(
            findings
                .iter()
                .any(|finding| finding.file.ends_with(file)
                    && finding.target.as_deref() == Some(target)),
            "missing {file}/{target}: {findings:?}"
        );
    }
    let merged = findings
        .iter()
        .filter(|finding| {
            finding.file.ends_with("job-merge.yml") && finding.target.as_deref() == Some("merged")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        merged.len(),
        1,
        "job merge must not prove an inherited write: {merged:?}"
    );
    assert!(merged[0].message.contains("cannot determine"));
    let jobs_merge = findings
        .iter()
        .filter(|finding| {
            finding.file.ends_with("jobs-merge.yml")
                && finding.target.as_deref() == Some("unanalyzable")
        })
        .collect::<Vec<_>>();
    assert_eq!(jobs_merge.len(), 1, "{jobs_merge:?}");
    assert!(jobs_merge[0].message.contains("merge"));
}

#[test]
fn custom_message_applies_to_definite_findings_and_unknown_advisories() {
    let (root, config, files) = fixture("custom");
    let sources = source_store_for_files(&files);
    let output =
        diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
    assert_eq!(output.findings.len(), 1, "{:?}", output.findings);
    assert_eq!(output.advisories.len(), 1, "{:?}", output.advisories);
    for (finding, target) in [
        (&output.findings[0], "build"),
        (&output.advisories[0], "unanalyzable"),
    ] {
        assert_eq!(finding.rule, RULE_ID);
        assert!(finding.file.ends_with("mixed.yml"));
        assert!(finding.line > 0);
        assert_eq!(finding.target.as_deref(), Some(target));
        assert_eq!(
            finding.message,
            "Move the write scope to the job that needs it."
        );
    }
}

#[test]
fn unreadable_utf8_and_missing_sources_follow_policy_and_suppression() {
    let (root, mut config, mut files) = fixture("unreadable");
    files.push(root.join(".github/workflows/missing.yml"));
    let sources = source_store_for_files(&files);
    let raw = diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
    assert_eq!(raw.findings.len(), 3, "{:?}", raw.findings);
    for (file, line) in [("invalid.yml", 2), ("disabled.yml", 3), ("missing.yml", 1)] {
        assert!(
            raw.findings
                .iter()
                .any(|finding| finding.file.ends_with(file)
                    && finding.line == line
                    && finding.target.as_deref() == Some("unanalyzable")),
            "missing {file}:{line}: {:?}",
            raw.findings
        );
    }
    let visible =
        diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
    assert_eq!(visible.findings.len(), 2, "{:?}", visible.findings);
    assert!(!visible
        .findings
        .iter()
        .any(|finding| finding.file.ends_with("disabled.yml")));
    config.rules[0].options = serde_yaml::from_str("unanalyzable: advise").unwrap();
    let advised =
        diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
    assert!(advised.findings.is_empty());
    assert_eq!(advised.advisories.len(), 2);
}

#[test]
fn unknown_job_permissions_locate_the_key_for_line_suppression() {
    let (root, config, files) = fixture("permission-suppression");
    let sources = source_store_for_files(&files);
    let raw = diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
    assert_eq!(raw.findings.len(), 4, "{:?}", raw.findings);
    for (target, line) in [("next-line", 7), ("same-line", 13), ("visible", 19)] {
        assert!(
            raw.findings
                .iter()
                .any(|finding| finding.file.ends_with("job-unknown.yml")
                    && finding.target.as_deref() == Some(target)
                    && finding.line == line),
            "missing {target}:{line}: {:?}",
            raw.findings
        );
    }
    let visible =
        diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
    assert_eq!(visible.findings.len(), 1, "{:?}", visible.findings);
    assert_eq!(visible.findings[0].target.as_deref(), Some("visible"));
}

#[test]
fn file_catalog_and_public_dispatch_keep_suppression_boundaries() {
    let (root, config, files) = fixture("suppression");
    // The file-catalog API defers suppression to its request owner.
    let raw = super::check_with_files(&root, &config, &files).unwrap();
    assert_eq!(raw.len(), 4, "{raw:?}");
    let dispatched =
        crate::codebase::rules::run_filesystem_rules_with_config(&root, &config, &files).unwrap();
    assert_eq!(dispatched, check(&root, &config).unwrap());
    assert_eq!(dispatched.len(), 1);
    assert_eq!(dispatched[0].target.as_deref(), Some("visible"));
}
