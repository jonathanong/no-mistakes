use super::*;

fn findings(yaml: &str) -> Vec<RuleFinding> {
    let root = unit_fixture("unanalyzable");
    let paths = vec![root.join("schema.sql"), root.join("write.ts")];
    let mut findings = check_with_files(&root, &config_with_options(yaml), &paths).unwrap();
    let sources = crate::codebase::rules::source_store_for_files(&paths);
    crate::codebase::rules::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    findings
}

fn lines(findings: &[RuleFinding]) -> Vec<usize> {
    findings.iter().map(|finding| finding.line).collect()
}

#[test]
fn dynamic_column_writes_fail_closed_by_default() {
    let findings = findings("{}");
    // The literal `logs` write, dynamic SELECT/DELETE and the suppressed call stay quiet.
    assert_eq!(lines(&findings), [5, 10, 15], "{findings:#?}");
    assert!(findings.iter().all(
        |finding| finding.target.as_deref() == Some("unanalyzable") && finding.import.is_none()
    ));
    let message = &findings[0].message;
    assert!(message.starts_with("write.ts:5: "), "{message}");
    assert!(message.contains("unanalyzableSql: ignore"), "{message}");
}

#[test]
fn ignore_skips_dynamic_sql() {
    assert!(findings("unanalyzableSql: ignore").is_empty());
}

#[test]
fn dynamic_sql_without_tracked_columns_stays_quiet() {
    let root = unit_fixture("unanalyzable");
    // Without the schema there is no generated or trigger-maintained column to protect.
    let findings =
        check_with_files(&root, &config_with_options("{}"), &[root.join("write.ts")]).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn unknown_unanalyzable_mode_is_a_config_error() {
    let root = unit_fixture("unanalyzable");
    let error = check_with_files(
        &root,
        &config_with_options("unanalyzableSql: sometimes"),
        &[root.join("write.ts")],
    )
    .err()
    .unwrap()
    .to_string();
    assert!(
        error.contains(
            "postgres-no-generated-column-writes: unanalyzableSql must be `fail` or `ignore`"
        ),
        "{error}"
    );
}
