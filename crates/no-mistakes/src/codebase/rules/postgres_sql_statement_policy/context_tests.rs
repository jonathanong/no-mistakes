use super::{expanded_tests::config, tests::fixture, *};

#[test]
fn procedural_prefixes_stop_before_case_expression_branches() {
    let root = fixture("expanded");
    let findings = check_with_files(&root, &config(true), &[root.join("prefixed.sql")]).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|f| (f.line, f.target.as_deref().unwrap()))
            .collect::<Vec<_>>(),
        vec![
            (1, "CREATE FUNCTION"),
            (2, "ALTER DATABASE"),
            (3, "ALTER SYSTEM"),
            (5, "CREATE PROCEDURE"),
            (6, "setting:session_replication_role"),
            (9, "setting:session_replication_role"),
            (12, "setting:session_replication_role"),
        ]
    );
}

#[test]
fn every_new_category_survives_do_function_and_static_execute_recovery() {
    let root = fixture("expanded");
    let findings =
        check_with_files(&root, &config(false), &[root.join("all-contexts.sql")]).unwrap();
    assert_eq!(findings.len(), 49, "{findings:?}");
    for kind in &crate::codebase::postgres::SUPPORTED_KINDS[7..] {
        let count = findings
            .iter()
            .filter(|finding| finding.target.as_deref() == Some(*kind))
            .count();
        assert_eq!(
            count,
            if *kind == "CREATE FUNCTION" { 4 } else { 3 },
            "{kind}: {findings:?}"
        );
    }
}

#[test]
fn settings_in_execute_and_guarded_routines_are_reported() {
    let root = fixture("expanded");
    let findings =
        check_with_files(&root, &config(true), &[root.join("setting-contexts.sql")]).unwrap();
    let settings: Vec<_> = findings
        .iter()
        .filter(|finding| finding.target.as_deref() == Some("setting:session_replication_role"))
        .map(|finding| finding.line)
        .collect();
    assert_eq!(settings, vec![2, 3, 6]);
}

#[test]
fn disabling_triggers_is_an_alter_table_statement() {
    let root = fixture("expanded");
    let findings =
        check_with_files(&root, &config(false), &[root.join("disable-trigger.sql")]).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("ALTER TABLE"));
}

#[test]
fn escaped_and_dollar_quoted_setting_names_are_static_literals() {
    let root = fixture("expanded");
    let findings =
        check_with_files(&root, &config(true), &[root.join("setting-literals.sql")]).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
}
