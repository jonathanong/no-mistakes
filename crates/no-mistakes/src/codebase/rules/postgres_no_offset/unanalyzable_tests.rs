use super::tests::{config_with_options, fixture};
use super::*;

fn findings(options: &str) -> Vec<RuleFinding> {
    let root = fixture("unanalyzable");
    let path = root.join("src/query.ts");
    let mut findings = check_with_files(
        &root,
        &config_with_options(options),
        std::slice::from_ref(&path),
    )
    .unwrap();
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    findings
}

fn lines_and_targets(findings: &[RuleFinding]) -> Vec<(usize, &str)> {
    findings
        .iter()
        .map(|finding| (finding.line, finding.target.as_deref().unwrap()))
        .collect()
}

#[test]
fn dynamic_select_or_opaque_sql_fails_closed_by_default() {
    let findings = findings("{}");
    // Dynamic INSERT (line 15), static SQL (line 25) and the suppressed call stay quiet.
    assert_eq!(
        lines_and_targets(&findings),
        [(5, "unanalyzable"), (10, "unanalyzable"), (20, "offset")],
        "{findings:#?}"
    );
    assert!(
        findings[0].message.contains("unanalyzableSql: ignore"),
        "{findings:#?}"
    );
    assert!(
        findings[0].message.starts_with("src/query.ts:5: "),
        "{findings:#?}"
    );
}

#[test]
fn ignore_keeps_only_proven_offsets() {
    let findings = findings("unanalyzableSql: ignore");
    assert_eq!(lines_and_targets(&findings), [(20, "offset")]);
    assert_eq!(
        lines_and_targets(&self::findings("unanalyzableSql: FAIL")),
        lines_and_targets(&self::findings("unanalyzableSql: fail"))
    );
}

#[test]
fn unknown_unanalyzable_mode_is_a_config_error() {
    let root = fixture("unanalyzable");
    let error = check_with_files(
        &root,
        &config_with_options("unanalyzableSql: sometimes"),
        &[root.join("src/query.ts")],
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("postgres-no-offset: unanalyzableSql must be `fail` or `ignore`"),
        "{error}"
    );
}
