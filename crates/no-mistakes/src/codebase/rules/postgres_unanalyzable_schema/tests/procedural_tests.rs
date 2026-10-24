use super::*;

#[test]
fn procedural_cursor_queries_retain_each_function_call_at_its_source_line() {
    for (name, expected) in [
        ("cursor-sql-facts.sql", &[5, 6, 11, 17, 27, 28, 33, 39][..]),
        ("cursor-first-parameter.sql", &[3, 14][..]),
        ("dml-query-loops.sql", &[8, 11, 14, 24, 27, 30][..]),
        (
            "with-query-calls.sql",
            &[6, 7, 10, 11, 18, 19, 22, 23, 28, 28, 28, 28, 29, 29, 29, 29][..],
        ),
    ] {
        let path = fixture().join("procedural/migrations").join(name);
        let sql = std::fs::read_to_string(path).unwrap();
        let facts = extract_migration_facts(&sql);
        assert!(
            facts.unanalyzable_statements.is_empty(),
            "{name}: {facts:#?}"
        );
        let mut lines: Vec<_> = facts
            .function_calls
            .iter()
            .filter(|call| call.name_parts == ["abs"])
            .map(|call| call.line)
            .collect();
        lines.sort_unstable();
        // Direct DECLARE AST facts and projected query facts must each appear once.
        assert_eq!(lines.as_slice(), expected, "{name}");
    }
}

#[test]
fn procedural_bodies_filter_commands_but_retain_unknown_sql_markers() {
    let root = fixture().join("procedural");
    let config = load_v2_config(&root, Some(&root.join("report.yml"))).unwrap();
    for (name, expected) in [
        ("valid.sql", &[][..]),
        (
            "unknown.sql",
            &[
                (6, "FROBULATE accounts"),
                (14, "FROBULATE first_after_if"),
                (22, "FROBULATE first_after_loop"),
            ][..],
        ),
        (
            "top-level.sql",
            &[
                (7, "FROBULATE after_do"),
                (13, "FROBULATE after_function"),
                (19, "FROBULATE after_procedure"),
                (21, "END IF"),
            ][..],
        ),
        ("quoted-do-bodies.sql", &[][..]),
        ("malformed-do-suffix.sql", &[(2, "DO")][..]),
        (
            "lexical-command-error.sql",
            &[
                (2, "DO"),
                (10, "FROBULATE after_lexical_body"),
                (13, "DO"),
                (19, "FROBULATE after_partial_lexical_body"),
            ][..],
        ),
        (
            "return-query.sql",
            &[(3, "RETURN QUERY FROBULATE accounts")][..],
        ),
        (
            "malformed-routine-headers.sql",
            &[(2, "CREATE FUNCTION"), (8, "CREATE PROCEDURE")][..],
        ),
        (
            "unknown-procedural-commands.sql",
            &[
                (4, "PERFORM 1 + )"),
                (10, "OPEN missing_cursor"),
                (15, "DECLARE bad_cursor"),
            ][..],
        ),
        (
            "unknown-payloads.sql",
            &[
                (7, "OPEN"),
                (8, "FROBULATE"),
                (9, "SELECT"),
                (10, "GET DIAGNOSTICS"),
                (11, "SELECT"),
            ][..],
        ),
        (
            "top-level-shapes.sql",
            &[(2, "DECLARE"), (3, "local_count")][..],
        ),
        ("unknown-loop-query.sql", &[(6, "FOR")][..]),
        ("declaration-cursor-variants.sql", &[][..]),
        ("cursor-sql-facts.sql", &[][..]),
        ("cursor-first-parameter.sql", &[][..]),
        ("dml-query-loops.sql", &[][..]),
        ("grammar-options.sql", &[][..]),
        ("with-query-calls.sql", &[][..]),
        (
            "grammar-errors.sql",
            &[
                (4, "DECLARE"),
                (6, "missing_type"),
                (7, "bad_cursor_prefix"),
                (8, "missing_cursor_for"),
                (9, "bad_collation"),
                (14, "1"),
                (15, "selected_id"),
                (16, "GET"),
                (17, "OPEN"),
                (18, "OPEN"),
                (19, "MOVE"),
                (20, "FETCH"),
                (21, "FOR"),
                (24, "FOR"),
                (27, "FOR"),
                (30, "FOR"),
                (33, "FOREACH"),
                (36, "FOREACH"),
                (44, "DECLARE"),
                (46, "missing_type"),
                (47, "wrong_collation"),
                (49, "GET"),
                (50, "OPEN"),
                (51, "OPEN"),
                (52, "OPEN"),
                (53, "OPEN"),
                (54, "MOVE"),
                (55, "MOVE"),
                (57, "FOR"),
                (58, "FOREACH"),
            ][..],
        ),
        ("malformed-command-tail.sql", &[(7, "OPEN")][..]),
        (
            "command-followups.sql",
            &[
                (11, "FROBULATE after_command_do"),
                (20, "FROBULATE after_command_function"),
            ][..],
        ),
        (
            "block-command-followups.sql",
            &[
                (11, "FROBULATE after_while"),
                (15, "FROBULATE after_for"),
                (19, "FROBULATE after_foreach"),
                (25, "FROBULATE after_exception"),
                (28, "FROBULATE after_block_do"),
                (39, "FROBULATE after_function_while"),
                (43, "FROBULATE after_function_for"),
                (47, "FROBULATE after_function_foreach"),
                (53, "FROBULATE after_function_exception"),
                (56, "FROBULATE after_block_function"),
            ][..],
        ),
    ] {
        let path = root.join("migrations").join(name);
        let sql = std::fs::read_to_string(&path).unwrap();
        let direct = extract_migration_facts(&sql).unanalyzable_statements;
        assert_eq!(direct.len(), expected.len(), "{name}: {direct:?}");
        for (marker, (line, head)) in direct.iter().zip(expected) {
            assert_eq!(marker.line, *line, "{name}");
            assert!(marker.head.starts_with(head), "{name}: {marker:?}");
        }

        let files = vec![path];
        let facts = crate::codebase::postgres::prepare_rule_sql_facts(
            &root,
            &files,
            source_store_for_files(&files),
            &config,
            RULE_IDS,
        )
        .unwrap();
        assert_eq!(
            facts
                .postgres_schema_file(&files[0])
                .unwrap()
                .unanalyzable_statements,
            direct
        );
        let findings = diagnostics_for_rule(&root, &config, &files, &facts, "postgres-fk-index")
            .unwrap()
            .findings;
        assert_eq!(findings.len(), direct.len(), "{name}: {findings:?}");
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.line)
                .collect::<Vec<_>>(),
            expected.iter().map(|(line, _)| *line).collect::<Vec<_>>(),
            "{name}"
        );
    }
}
