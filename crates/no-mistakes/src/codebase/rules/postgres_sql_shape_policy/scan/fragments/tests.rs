use crate::codebase::rules::postgres_sql_shape_policy::{check_with_files, RULE_ID};
use crate::config::v2::load_v2_config;
use std::path::PathBuf;

fn variant(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-shape-policy/variants")
        .join(name)
}

fn findings(name: &str) -> Vec<crate::codebase::rules::RuleFinding> {
    let root = variant(name);
    let config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files = [root.join("src/query.ts")];
    check_with_files(&root, &config, &files).unwrap()
}

#[test]
fn complete_execution_filter_keeps_unrelated_fragment_origins() {
    let findings = findings("executed-complete-owned");
    let lines = findings
        .iter()
        .map(|finding| finding.line)
        .collect::<Vec<_>>();
    assert_eq!(lines, [2, 7], "{findings:?}");
    assert!(findings.iter().all(|finding| finding.rule == RULE_ID
        && finding.target.as_deref() == Some("banned-function-call")));
}

#[test]
fn host_suppressed_finite_fragments_still_scan_the_disabled_host() {
    let findings = findings("finite-fragments-host-suppressed");
    assert!(
        findings.iter().any(|finding| finding.line == 4
            && finding.target.as_deref() == Some("banned-function-call")),
        "{findings:?}"
    );
}

#[test]
fn unexecuted_finite_fragments_keep_their_origin_lines() {
    let findings = findings("finite-fragments-unexecuted");
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(findings[0].line, 2);
    assert!(
        findings[0].message.starts_with("src/query.ts:2:"),
        "{}",
        findings[0].message
    );
    assert_eq!(findings[1].line, 10);
    assert!(
        findings[1]
            .message
            .starts_with("src/query.ts:10: builder SQL"),
        "{}",
        findings[1].message
    );
}
