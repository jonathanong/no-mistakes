use super::*;
use sqlparser::tokenizer::Tokenizer;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/ddl-compat/body-recovery")
            .join(name),
    )
    .unwrap()
}

#[test]
fn loop_actions_keep_their_query_ast_without_source_locations() {
    let sql = fixture("unlocated-loop-actions.sql");
    let located = Tokenizer::new(&PostgreSqlDialect {}, &sql)
        .tokenize_with_location()
        .unwrap();
    for original in [false, true] {
        let mut actions = 0;
        for chunk in located.split(|token| token.token == Token::SemiColon) {
            if chunk
                .iter()
                .all(|token| matches!(token.token, Token::Whitespace(_)))
            {
                continue;
            }
            let tokens: Vec<_> = chunk.iter().map(|token| token.token.clone()).collect();
            let recovered = recover(&tokens, original.then_some(chunk), true);
            assert_eq!(recovered.len(), 1, "{recovered:#?}");
            let (statement, synthetic) = &recovered[0];
            assert!(*synthetic && matches!(statement, Statement::Query(_)));
            let mut calls = Vec::new();
            crate::codebase::postgres::function_calls::collect_projected(
                statement, &mut calls, *synthetic,
            );
            assert_eq!(
                calls.len(),
                1,
                "the template/header is not the action: {calls:#?}"
            );
            assert_eq!(calls[0].name_parts, ["abs"]);
            actions += 1;
        }
        assert_eq!(actions, 3);
    }
}

#[test]
fn invalid_strict_targets_cannot_be_recovered_as_valid_query_actions() {
    let sql = fixture("invalid-strict-query-actions.sql");
    let located = Tokenizer::new(&PostgreSqlDialect {}, &sql)
        .tokenize_with_location()
        .unwrap();
    for chunk in located.split(|token| token.token == Token::SemiColon) {
        let tokens: Vec<_> = chunk.iter().map(|token| token.token.clone()).collect();
        assert!(recover(&tokens, Some(chunk), true).is_empty());
    }
}

#[test]
fn newline_continuation_cannot_join_a_single_quote_and_dollar_quote() {
    let sql = fixture("mixed-comment-literal.sql");
    assert!(crate::codebase::postgres::parse_postgres_sql(&sql).is_err());
    assert!(
        !crate::codebase::postgres::migration::extract_migration_facts(&sql)
            .unanalyzable_statements
            .is_empty()
    );
}
