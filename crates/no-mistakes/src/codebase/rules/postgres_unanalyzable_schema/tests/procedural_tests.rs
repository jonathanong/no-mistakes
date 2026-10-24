use super::*;

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
                (4, "PERFORM FROBULATE"),
                (10, "OPEN missing_cursor"),
                (15, "DECLARE bad_cursor"),
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
