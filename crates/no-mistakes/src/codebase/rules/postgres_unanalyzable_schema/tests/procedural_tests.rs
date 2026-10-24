use super::*;

#[test]
fn peeled_do_filters_only_its_end_if_marker() {
    let root = fixture().join("procedural");
    let config = load_v2_config(&root, Some(&root.join("report.yml"))).unwrap();
    for (name, expected_line, expected_head) in [
        ("valid.sql", None, ""),
        ("unknown.sql", Some(6), "FROBULATE accounts"),
        ("top-level.sql", Some(2), "END IF"),
    ] {
        let path = root.join("migrations").join(name);
        let sql = std::fs::read_to_string(&path).unwrap();
        let direct = extract_migration_facts(&sql).unanalyzable_statements;
        assert_eq!(
            direct.len(),
            usize::from(expected_line.is_some()),
            "{name}: {direct:?}"
        );
        if let Some(line) = expected_line {
            assert_eq!(direct[0].line, line, "{name}");
            assert_eq!(direct[0].head, expected_head, "{name}");
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
        if let Some(line) = expected_line {
            assert_eq!(findings[0].line, line, "{name}");
        }
    }
}
