use super::*;

#[test]
fn copy_data_and_terminators_do_not_report_but_following_sql_does() {
    let root = fixture().join("copy");
    let sql = std::fs::read_to_string(root.join("migrations/001.sql")).unwrap();
    let direct = extract_migration_facts(&sql).unanalyzable_statements;
    assert_eq!(direct.len(), 1, "{direct:?}");
    assert_eq!(direct[0].line, 8);
    assert!(direct[0].head.starts_with("FROBULATE accounts"));
    assert_eq!(direct[0].source_order, [2]);

    let (config, files, facts) = prepared(&root, "report.yml");
    assert_eq!(
        &facts
            .postgres_schema_file(&files[0])
            .unwrap()
            .unanalyzable_statements,
        &direct
    );
    let findings = diagnostics_for_rule(&root, &config, &files, &facts, "postgres-fk-index")
        .unwrap()
        .findings;
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 8);
}

#[test]
fn static_execute_after_copy_does_not_leave_wrapper_marker() {
    let root = fixture().join("copy");
    let sql = std::fs::read_to_string(root.join("migrations/static-execute.sql")).unwrap();
    let facts = extract_migration_facts(&sql);
    assert!(facts.unanalyzable_statements.is_empty(), "{facts:?}");
    assert!(facts
        .tables
        .iter()
        .any(|table| table.table_name == "audit_log"));
}

#[test]
fn recovered_copy_inside_do_does_not_consume_outer_backslash_statement() {
    let root = fixture().join("copy");
    let sql = std::fs::read_to_string(root.join("migrations/procedural.sql")).unwrap();
    let markers = extract_migration_facts(&sql).unanalyzable_statements;
    assert_eq!(markers.len(), 1, "{markers:?}");
    assert_eq!(markers[0].line, 3);
}

#[test]
fn unsupported_copy_header_owns_data_and_terminator() {
    let root = fixture().join("copy");
    let sql = std::fs::read_to_string(root.join("migrations/unsupported.sql")).unwrap();
    let markers = extract_migration_facts(&sql).unanalyzable_statements;
    assert_eq!(markers.len(), 2, "{markers:?}");
    assert_eq!(
        markers.iter().map(|marker| marker.line).collect::<Vec<_>>(),
        [2, 5]
    );
    assert!(markers[0].head.starts_with("COPY accounts"));
    assert!(markers[1].head.starts_with("FROBULATE accounts"));
    assert_eq!(markers[0].source_order, [0]);
    assert_eq!(markers[1].source_order, [1]);
}

#[test]
fn copy_terminator_skip_does_not_hide_unrelated_backslash_statement() {
    let root = fixture().join("copy");
    let sql = std::fs::read_to_string(root.join("migrations/standalone.sql")).unwrap();
    let markers = extract_migration_facts(&sql).unanalyzable_statements;
    assert_eq!(markers.len(), 1, "{markers:?}");
    assert_eq!(markers[0].line, 1);
}

#[test]
fn crlf_copy_terminator_keeps_following_statement_line() {
    let root = fixture().join("copy");
    let sql = std::fs::read_to_string(root.join("migrations/crlf.sql")).unwrap();
    assert!(sql.contains("\\.\r\n"));
    let markers = extract_migration_facts(&sql).unanalyzable_statements;
    assert_eq!(markers.len(), 1, "{markers:?}");
    assert_eq!(markers[0].line, 4);
    assert!(markers[0].head.starts_with("FROBULATE accounts"));
}
