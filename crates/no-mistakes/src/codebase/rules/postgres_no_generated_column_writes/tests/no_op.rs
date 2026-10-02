use super::*;

#[test]
fn if_not_exists_preserves_existing_state_and_accepts_missing_definitions() {
    let root = unit_fixture("if-not-exists");
    let findings = check_with_files(
        &root,
        &config_with_options("triggerMaintainedColumns: [updated_at]"),
        &[root.join("schema.sql"), root.join("writes.sql")],
    )
    .unwrap();
    assert_eq!(findings.len(), 5, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [1, 2, 3, 3, 4]
    );
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.message.contains("trigger-maintained column"))
            .count(),
        2
    );
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.message.contains("generated column"))
            .count(),
        3
    );
}

#[test]
fn qualified_lifecycles_do_not_replace_other_schemas_and_ambiguous_names_are_skipped() {
    let root = unit_fixture("qualified-lifecycle");
    let findings = check_with_files(
        &root,
        &config_with_options(""),
        &[root.join("schema.sql"), root.join("writes.sql")],
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [1, 3]
    );
    assert!(findings
        .iter()
        .all(|finding| finding.import.as_deref() == Some("public.orders.generated")));
}

#[test]
fn live_catalog_replays_numeric_migration_names_in_numeric_order() {
    let root = unit_fixture("migration-order");
    let findings = check_with_files(
        &root,
        &config_with_options(""),
        &[
            root.join("10.sql"),
            root.join("2.sql"),
            root.join("writes.sql"),
        ],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].import.as_deref(), Some("orders.generated"));
}

#[test]
fn dormant_and_conditional_routine_ddl_does_not_remove_live_generated_columns() {
    let root = unit_fixture("table-event-forms");
    let findings = check_with_files(
        &root,
        &config_with_options("sqlInclude: ['**/execution.sql']"),
        &[
            root.join("execution.sql"),
            root.join("execution-writes.sql"),
        ],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].import.as_deref(), Some("orders.computed"));
}

#[test]
fn collected_empty_event_stream_does_not_fall_back_to_dormant_schema_metadata() {
    let root = unit_fixture("table-event-forms");
    let findings = check_with_files(
        &root,
        &config_with_options("sqlInclude: ['**/dormant-only.sql']"),
        &[
            root.join("dormant-only.sql"),
            root.join("dormant-writes.sql"),
        ],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
    let source = std::fs::read_to_string(root.join("dormant-only.sql")).unwrap();
    let facts = crate::codebase::postgres::extract_migration_facts(&source);
    assert!(facts.table_events_collected);
    assert!(facts.table_events.is_empty());
    assert!(
        !facts.tables.is_empty(),
        "Policy metadata must still include dormant DDL"
    );
}

#[test]
fn conditional_alter_does_not_revive_dropped_or_absent_tables() {
    let root = unit_fixture("conditional-alter");
    let findings = check_with_files(
        &root,
        &config_with_options(""),
        &[root.join("schema.sql"), root.join("writes.sql")],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 3);
    assert_eq!(findings[0].import.as_deref(), Some("orders.computed"));
}

#[test]
fn global_and_local_temporary_creates_keep_source_order() {
    let root = unit_fixture("global-local");
    let findings = check_with_files(
        &root,
        &config_with_options(""),
        &[root.join("schema.sql"), root.join("writes.sql")],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 2);
    assert_eq!(
        findings[0].import.as_deref(),
        Some("reverse_orders.computed")
    );
}
