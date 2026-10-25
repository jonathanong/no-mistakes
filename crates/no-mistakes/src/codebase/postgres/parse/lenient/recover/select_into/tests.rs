#[test]
fn with_insert_query_payload_keeps_cte_calls() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-ddl-unanalyzable/fixture/procedural/migrations/with-query-calls.sql"));
    let facts = crate::codebase::postgres::migration::extract_migration_facts(sql);
    assert!(facts.unanalyzable_statements.is_empty(), "{facts:#?}");
    // Both a CTE before the INSERT and same-line siblings must survive.
    for (line, expected) in [(6, 1), (18, 1), (28, 4), (29, 4)] {
        assert_eq!(
            facts
                .function_calls
                .iter()
                .filter(|call| call.name_parts == ["abs"] && call.line == line)
                .count(),
            expected,
            "line {line}: {facts:#?}"
        );
    }
}

fn nested_fixture(name: &str) -> String {
    std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/ddl-compat/body-recovery")
            .join(name),
    )
    .unwrap()
}

#[test]
fn nested_target_commas_and_from_keep_complete_body_queries_and_source_facts() {
    use crate::codebase::postgres::parse::parse_postgres_body_with_function_sources;
    use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, tokenizer::Tokenizer};
    let sql = nested_fixture("strict-nested-targets.sql");
    let source = Tokenizer::new(&PostgreSqlDialect {}, &sql)
        .tokenize_with_location()
        .unwrap();
    let parsed = parse_postgres_body_with_function_sources(&sql);
    assert_eq!(parsed.len(), 2, "{parsed:#?}");
    for (index, item) in parsed.iter().enumerate() {
        assert!(
            item.complete && item.recovery_failure.is_none(),
            "{item:#?}"
        );
        let tokens = item.chunk_tokens.as_ref().unwrap();
        assert!(tokens.iter().all(|token| source.contains(token)));
        assert!(tokens
            .iter()
            .any(|token| token.token.to_string() == "STRICT"));
        let Statement::Query(query) = &item.statement else {
            panic!("{item:#?}")
        };
        let sqlparser::ast::SetExpr::Select(select) = query.body.as_ref() else {
            panic!("{query:#?}")
        };
        assert!(select.selection.is_some());
        assert_eq!(select.projection.len(), 1);
        assert_eq!(select.from.len(), 1);
        let into = select.into.as_ref().unwrap();
        assert_eq!(into.targets.len(), index + 1);
        // Internal commas and EXTRACT's FROM belong to the subscript expression.
        assert_eq!(
            into.targets[0].to_string(),
            [
                "result_ids[array_position(ARRAY[1, 2], 1)]",
                "result_ids[EXTRACT(DAY FROM now())::INT]"
            ][index]
        );
        if index == 1 {
            assert_eq!(into.targets[1].to_string(), "second_id");
        }
    }
    let facts = crate::codebase::postgres::migration::extract_migration_body_facts(&sql);
    assert!(facts.unanalyzable_statements.is_empty(), "{facts:#?}");
    for (index, line) in sql.lines().enumerate() {
        for name in ["abs", "array_position", "now"] {
            if let Some(column) = line.find(&format!("{name}(")) {
                assert!(
                    facts
                        .function_calls
                        .iter()
                        .any(|call| call.name_parts == [name]
                            && call.line == index + 1
                            && call.column == column + 1),
                    "missing {name}: {facts:#?}"
                );
            }
        }
    }
    // Compatibility remains scoped to procedural bodies, including this exact shape.
    assert!(crate::codebase::postgres::parse_postgres_sql(&sql).is_err());
}

#[test]
fn nested_target_validation_rejects_bad_delimiters_empty_targets_and_non_lvalues() {
    use sqlparser::{dialect::PostgreSqlDialect, tokenizer::Tokenizer};
    for line in nested_fixture("invalid-strict-nested-targets.sql").lines() {
        let mut tokens = Tokenizer::new(&PostgreSqlDialect {}, line)
            .tokenize_with_location()
            .unwrap();
        assert!(!super::normalize(&mut tokens), "{line}");
    }
}
