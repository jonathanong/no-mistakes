use super::*;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::Tokenizer;

#[test]
fn sql_bearing_control_chunks_stay_inside_each_procedural_wrapper() {
    for wrapper in ["do", "function", "procedure"] {
        let path = crate::test_support::rule_fixture_root(
            "../check-runner/postgres-parse-failure-locations",
        )
        .join(format!("procedural-control-{wrapper}.sql"));
        let sql = std::fs::read_to_string(path).unwrap();
        let facts = crate::codebase::postgres::migration::extract_migration_facts(&sql);
        assert!(
            facts
                .function_calls
                .iter()
                .filter(|call| call.name_parts == ["abs"])
                .count()
                >= 2,
            "SQL inside loops and PERFORM retains function facts: {wrapper}: {facts:#?}"
        );
        let malformed: Vec<_> = sql
            .lines()
            .enumerate()
            .filter(|(_, line)| matches!(*line, "SELECT FROM;" | "ALTER TABLE;"))
            .map(|(line, _)| line + 1)
            .collect();
        assert_eq!(
            facts
                .unanalyzable_statements
                .iter()
                .map(|failure| failure.line)
                .collect::<Vec<_>>(),
            malformed,
            "{wrapper}: {facts:#?}"
        );
        assert!(facts
            .tables
            .iter()
            .any(|table| table.table_name == "following_body"));
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, &sql)
            .tokenize_with_location()
            .unwrap();
        let body = tokens
            .iter()
            .find_map(|token| match &token.token {
                Token::DollarQuotedString(body) => Some(&body.value),
                _ => None,
            })
            .unwrap();
        let body = Tokenizer::new(&PostgreSqlDialect {}, body)
            .tokenize_with_location()
            .unwrap();
        let mut state = BodyState::default();
        let controls: Vec<_> = body
            .split(|token| token.token == Token::SemiColon)
            .filter(|chunk| {
                chunk
                    .iter()
                    .any(|token| !matches!(token.token, Token::Whitespace(_)))
            })
            .map(|chunk| state.control_chunk(chunk))
            .collect();
        assert!(
            controls.iter().all(|control| *control),
            "{wrapper}: {controls:?}"
        );
    }
}

#[test]
fn malformed_sql_bearing_commands_keep_their_failure() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("../check-runner/postgres-parse-failure-locations")
            .join("procedural-control-malformed.sql"),
    )
    .unwrap();
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, &sql)
        .tokenize_with_location()
        .unwrap();
    let mut state = BodyState::default();
    for chunk in tokens.split(|token| token.token == Token::SemiColon) {
        if chunk
            .iter()
            .any(|token| !matches!(token.token, Token::Whitespace(_)))
        {
            assert!(!state.control_chunk(chunk), "{chunk:#?}");
        }
    }
}

#[test]
fn execute_result_targets_and_parameters_require_complete_expressions() {
    for (fixture, expected) in [
        ("procedural-execute-targets-valid.sql", true),
        ("procedural-execute-targets-malformed.sql", false),
    ] {
        let sql = std::fs::read_to_string(
            crate::test_support::rule_fixture_root(
                "../check-runner/postgres-parse-failure-locations",
            )
            .join(fixture),
        )
        .unwrap();
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, &sql)
            .tokenize_with_location()
            .unwrap();
        let mut state = BodyState::default();
        for chunk in tokens.split(|token| token.token == Token::SemiColon) {
            if chunk
                .iter()
                .any(|token| !matches!(token.token, Token::Whitespace(_)))
            {
                assert_eq!(
                    state.control_chunk(chunk),
                    expected,
                    "{fixture}: {chunk:#?}"
                );
            }
        }
    }
}

#[test]
fn perform_key_share_recovery_keeps_function_identity_and_source_span() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("../check-runner/postgres-parse-failure-locations")
            .join("procedural-perform-lock-projection.sql"),
    )
    .unwrap();
    let original = Tokenizer::new(&PostgreSqlDialect {}, &sql)
        .tokenize_with_location()
        .unwrap();
    let original = original
        .split(|token| token.token == Token::SemiColon)
        .next()
        .unwrap();
    let tokens: Vec<_> = original.iter().map(|token| token.token.clone()).collect();
    for located in [true, false] {
        let recovered = super::super::queries::recover(&tokens, located.then_some(original), true);
        assert_eq!(recovered.len(), 1, "{recovered:#?}");
        let (statement, synthetic) = &recovered[0];
        let mut calls = Vec::new();
        crate::codebase::postgres::function_calls::collect_projected(
            statement, &mut calls, *synthetic,
        );
        assert_eq!(calls.len(), 1, "{calls:#?}");
        assert_eq!(calls[0].name_parts, ["abs"]);
        if located {
            assert_eq!((calls[0].line, calls[0].column), (2, 9));
        }
    }
}

#[test]
fn plain_locked_queries_keep_complete_asts_and_function_occurrences() {
    for wrapper in ["do", "function", "procedure"] {
        let sql = std::fs::read_to_string(
            crate::test_support::rule_fixture_root(
                "../check-runner/postgres-parse-failure-locations",
            )
            .join(format!("procedural-plain-query-{wrapper}.sql")),
        )
        .unwrap();
        let facts = crate::codebase::postgres::migration::extract_migration_facts(&sql);
        let expected: Vec<_> = sql
            .lines()
            .enumerate()
            .filter_map(|(line, source)| {
                if source.starts_with("    SELECT") || source.starts_with("    WITH") {
                    Some(("abs", line + 1, source.find("abs").unwrap() + 1))
                } else {
                    None
                }
            })
            .collect();
        for (name, line, column) in &expected {
            assert!(
                facts
                    .function_calls
                    .iter()
                    .any(|call| call.name_parts == [*name]
                        && call.line == *line
                        && call.column == *column),
                "{wrapper}: missing {name} at {line}:{column}: {facts:#?}"
            );
        }
        let malformed = sql
            .lines()
            .position(|line| line.contains("FROBULATE"))
            .unwrap()
            + 1;
        assert_eq!(
            facts
                .unanalyzable_statements
                .iter()
                .map(|failure| failure.line)
                .collect::<Vec<_>>(),
            [malformed],
            "{wrapper}: {facts:#?}"
        );
        assert!(facts
            .tables
            .iter()
            .any(|table| table.table_name == "following_locked_query"));
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, &sql)
            .tokenize_with_location()
            .unwrap();
        let body = tokens
            .iter()
            .find_map(|token| match &token.token {
                Token::DollarQuotedString(body) => Some(&body.value),
                _ => None,
            })
            .unwrap();
        // All wrappers share the same body parser; inspect its ordinary query projection.
        let parsed = super::super::super::parse_with_sources(body, true, false, true);
        let queries: Vec<_> = parsed
            .iter()
            .filter(|item| matches!(item.statement, Statement::Query(_)))
            .collect();
        assert_eq!(queries.len(), 2, "{wrapper}: {parsed:#?}");
        for query in queries {
            assert!(
                query.complete && !query.function_projection && query.recovery_failure.is_none(),
                "{query:#?}"
            );
            let Statement::Query(query) = &query.statement else {
                unreachable!()
            };
            let sqlparser::ast::SetExpr::Select(select) = query.body.as_ref() else {
                panic!("expected SELECT: {query:#?}")
            };
            assert!(select.into.is_some());
            assert_eq!(query.locks.len(), 1);
        }
    }
}

#[path = "body_recovery_tests.rs"]
mod body_recovery_tests;

#[test]
fn literal_execute_loop_headers_require_complete_commands() {
    let path = crate::test_support::rule_fixture_root("postgres-ddl-unanalyzable")
        .join("loop-policy/malformed.sql");
    for line in std::fs::read_to_string(path).unwrap().lines() {
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, line)
            .tokenize_with_location()
            .unwrap();
        assert!(!BodyState::default().control_chunk(&tokens), "{line}");
    }
}
