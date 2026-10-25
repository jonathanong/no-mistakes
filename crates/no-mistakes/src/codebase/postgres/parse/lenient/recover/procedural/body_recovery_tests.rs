use crate::codebase::postgres::{migration::extract_migration_facts, parse_postgres_sql};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::{Token, Tokenizer};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/ddl-compat/body-recovery")
            .join(name),
    )
    .unwrap()
}

#[test]
fn body_recovery_retains_strict_targets_escaped_dynamic_loops_and_locked_queries() {
    for wrapper in ["do", "function", "procedure"] {
        let sql = fixture(&format!("{wrapper}.sql"));
        let facts = extract_migration_facts(&sql);
        assert!(
            facts.unanalyzable_statements.is_empty(),
            "{wrapper}: {facts:#?}"
        );
        for (line, text) in sql
            .lines()
            .enumerate()
            .filter(|(_, text)| text.contains("abs(") && !text.contains("EXECUTE"))
        {
            assert!(
                facts
                    .function_calls
                    .iter()
                    .any(|call| call.name_parts == ["abs"]
                        && call.line == line + 1
                        && call.column == text.find("abs(").unwrap() + 1),
                "missing function at {}: {facts:#?}",
                line + 1
            );
        }
        assert_eq!(
            facts
                .function_calls
                .iter()
                .filter(|call| call.name_parts == ["abs"])
                .count(),
            7,
            "{facts:#?}"
        );
        let dynamic_line = sql
            .lines()
            .position(|line| line.contains("EXECUTE format"))
            .unwrap()
            + 1;
        assert!(
            facts
                .function_calls
                .iter()
                .any(|call| call.name_parts == ["abs"] && call.line == dynamic_line),
            "dynamic template lost its SQL function: {facts:#?}"
        );
        assert!(facts
            .tables
            .iter()
            .any(|table| table.table_name == "following_body_recovery"));
    }
}

#[test]
fn comments_preserve_ast_text_for_adjacent_and_dollar_literals() {
    let sql = fixture("comments.sql");
    let parsed = parse_postgres_sql(&sql).unwrap();
    assert_eq!(parsed.len(), 6);
    assert!(parsed[0].to_string().contains("a. b."), "{:?}", parsed[0]);
    assert!(extract_migration_facts(&sql)
        .unanalyzable_statements
        .is_empty());
}

#[test]
fn malformed_top_level_and_procedural_controls_remain_failures() {
    for line in fixture("malformed.sql").lines() {
        assert!(
            parse_postgres_sql(line).is_err(),
            "accepted malformed SQL: {line}"
        );
        assert!(
            !extract_migration_facts(line)
                .unanalyzable_statements
                .is_empty(),
            "lost malformed SQL: {line}"
        );
    }
    let sql = fixture("malformed-body.sql");
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, &sql)
        .tokenize_with_location()
        .unwrap();
    let mut state = super::super::BodyState::default();
    for chunk in tokens
        .split(|token| token.token == Token::SemiColon)
        .filter(|chunk| {
            chunk
                .iter()
                .any(|token| !matches!(token.token, Token::Whitespace(_)))
        })
    {
        assert!(
            !state.control_chunk(chunk),
            "accepted malformed control: {chunk:#?}"
        );
    }
}

#[test]
fn mixed_lock_normalization_keeps_query_asts_and_spans() {
    let sql = fixture("mixed-locks.sql");
    let original = Tokenizer::new(&PostgreSqlDialect {}, &sql)
        .tokenize_with_location()
        .unwrap();
    let mut tokens = original.clone();
    super::super::super::query_locks::rewrite_located(&mut tokens);
    assert!(tokens.iter().all(|token| original.contains(token)));
    let statements = sqlparser::parser::Parser::new(&PostgreSqlDialect {})
        .with_tokens_with_locations(tokens)
        .parse_statements()
        .unwrap();
    let sqlparser::ast::Statement::Query(query) = &statements[0] else {
        panic!("expected query")
    };
    assert_eq!(query.locks.len(), 2);
    assert!(query.order_by.is_some());
    let sqlparser::ast::SetExpr::Select(select) = query.body.as_ref() else {
        panic!("expected select")
    };
    assert!(select.selection.is_some());
    assert_eq!(select.projection.len(), 2);
}

#[test]
fn strict_into_uses_only_the_analysis_clone_and_keeps_complete_query_asts() {
    let sql = fixture("do.sql");
    let outer = Tokenizer::new(&PostgreSqlDialect {}, &sql)
        .tokenize_with_location()
        .unwrap();
    let body = outer
        .iter()
        .find_map(|token| match &token.token {
            Token::DollarQuotedString(body) => Some(&body.value),
            _ => None,
        })
        .unwrap();
    let parsed = crate::codebase::postgres::parse::parse_postgres_body_with_function_sources(body);
    let queries: Vec<_> = parsed
        .iter()
        .filter(|item| {
            !item.function_projection
                && matches!(item.statement, sqlparser::ast::Statement::Query(_))
        })
        .collect();
    assert_eq!(queries.len(), 2);
    for (index, item) in queries.into_iter().enumerate() {
        assert!(
            item.complete && item.recovery_failure.is_none(),
            "{item:#?}"
        );
        let tokens = item.chunk_tokens.as_ref().unwrap();
        assert!(tokens
            .iter()
            .any(|token| token.token.to_string() == "STRICT"));
        let sqlparser::ast::Statement::Query(query) = &item.statement else {
            unreachable!()
        };
        let sqlparser::ast::SetExpr::Select(select) = query.body.as_ref() else {
            panic!("expected select")
        };
        assert!(select.selection.is_some());
        let into = select.into.as_ref().unwrap();
        assert_eq!(into.targets.len(), index + 1);
        assert_eq!(into.targets[0].to_string(), "first_id");
        if index == 1 {
            assert_eq!(into.targets[1].to_string(), "second_id");
        }
    }
}
