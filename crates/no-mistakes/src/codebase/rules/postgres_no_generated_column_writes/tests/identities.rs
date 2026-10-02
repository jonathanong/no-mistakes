use super::*;

fn lines(name: &str) -> Vec<usize> {
    let root = unit_fixture(name);
    check_with_files(
        &root,
        &config_with_options(""),
        &[root.join("schema.sql"), root.join("writes.sql")],
    )
    .unwrap()
    .into_iter()
    .map(|finding| finding.line)
    .collect()
}

#[test]
fn quoted_dots_and_case_remain_distinct_from_qualification() {
    assert_eq!(lines("quoted-identity"), [1, 3, 5, 7]);
}

#[test]
fn one_embedded_call_retains_distinct_quoted_case_identities() {
    let root = unit_fixture("quoted-identity");
    let findings = check_with_files(
        &root,
        &config_with_options(""),
        &[root.join("schema.sql"), root.join("write.ts")],
    )
    .unwrap();
    let imports: Vec<_> = findings
        .iter()
        .map(|finding| finding.import.as_deref().unwrap())
        .collect();
    assert_eq!(imports, ["\"ORDERS\".computed", "\"Orders\".computed"]);
}

#[test]
fn dropping_a_temporary_table_reveals_the_permanent_table() {
    assert_eq!(lines("temporary-drop"), [1, 2, 3]);
}

#[test]
fn temporary_tables_shadow_only_unqualified_lookups() {
    assert_eq!(lines("temporary-shadow"), [2, 3, 6]);
}

#[test]
fn exception_handlers_do_not_remove_definitely_live_tables() {
    assert_eq!(lines("exception-handlers"), [1]);
}

#[test]
fn rollback_restores_tables_and_discards_aborted_columns() {
    assert_eq!(lines("transaction-rollback"), [1, 4]);
}

#[test]
fn conditional_variable_assignments_do_not_change_definite_live_history() {
    assert_eq!(lines("conditional-assignment"), [1]);
    let root = unit_fixture("conditional-assignment");
    let sql = std::fs::read_to_string(root.join("schema.sql")).unwrap();
    let facts = crate::codebase::postgres::extract_migration_facts(&sql);
    assert!(facts
        .tables
        .iter()
        .any(|table| table.table_name == "dormant_fact"));
    assert!(facts
        .dropped_tables
        .iter()
        .any(|drop| drop.name == "orders"));
    assert!(facts.table_events.iter().all(|event| !matches!(
        event,
        crate::codebase::postgres::SqlTableSchemaEvent::Drop { .. }
    )));
}

#[test]
fn configured_extras_follow_temp_unique_and_qualified_relation_identity() {
    let root = unit_fixture("configured-identities");
    let options = std::fs::read_to_string(root.join("options.yml")).unwrap();
    let findings = check_with_files(
        &root,
        &config_with_options(&options),
        &[root.join("schema.sql"), root.join("writes.sql")],
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 7, 8, 9, 10, 11]
    );
    assert!(findings
        .iter()
        .all(|finding| !finding.message.contains("stale")));
}
