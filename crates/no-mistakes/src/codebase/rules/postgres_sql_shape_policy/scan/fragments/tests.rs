use crate::codebase::rules::postgres_sql_shape_policy::{check_with_files, RULE_ID};
use crate::config::v2::load_v2_config;
use std::path::PathBuf;

// Output cases moved to cli_postgres_snapshot_sql_shape_policy.

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
fn disabled_complete_execution_drops_only_its_own_append_site() {
    // The disabled execution is the only append of this token, so its fragment
    // finding must not leak. The call-site finding stays for suppression accounting.
    let owned = findings("complete-suppression-owned");
    let owned_lines = owned.iter().map(|finding| finding.line).collect::<Vec<_>>();
    assert_eq!(owned_lines, [7], "{owned:#?}");

    // The same token also feeds an unrelated builder. That finding stays on the
    // token line while the disabled call stays on its own line.
    let reuse = findings("complete-suppression-reuse");
    let reuse_lines = reuse.iter().map(|finding| finding.line).collect::<Vec<_>>();
    assert_eq!(reuse_lines, [2, 5], "{reuse:#?}");
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
