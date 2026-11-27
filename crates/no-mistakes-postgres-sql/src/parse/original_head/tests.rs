use super::*;
use sqlparser::{dialect::PostgreSqlDialect, tokenizer::Tokenizer};

#[test]
fn mismatched_source_views_preserve_the_entire_existing_failure() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-ddl-unanalyzable")
            .join("issue-2731/migrations/radix-heads.sql"),
    )
    .unwrap();
    let statements = super::super::parse_postgres_sql_lenient_with_sources(&sql, &sql);
    let before: Vec<_> = statements
        .iter()
        .map(|item| item.recovery_failure.clone())
        .collect();
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, &sql)
        .tokenize_with_location()
        .unwrap();
    // Change one end at a time, including mid-head and radix-tail tokens.
    // Starts stay sorted, so restoration must reach each affected token rather
    // than rejecting the whole inventory at its first anchor.
    for at in 0..tokens.len() {
        let mut drifted = tokens.clone();
        drifted[at].span.end.line = 999;
        let mut statements = super::super::parse_postgres_sql_lenient_with_sources(&sql, &sql);
        OriginalHeads::capture(&drifted).restore(
            &sql,
            &sql,
            &sql,
            &HeadSourceMap::default(),
            &mut statements,
        );
        let after: Vec<_> = statements
            .iter()
            .map(|item| item.recovery_failure.clone())
            .collect();
        assert_eq!(after, before, "drifted token {at}");
    }
}

#[test]
fn synthetic_grouping_separators_keep_original_head_lexemes() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-ddl-unanalyzable")
            .join("issue-2731/migrations/group-rewrite-heads.sql"),
    )
    .unwrap();
    let (tokenized, map) = super::super::distinct_group::separate_with_head_map(&sql);
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, &tokenized)
        .tokenize_with_location()
        .unwrap();
    let mut statements = super::super::parse_postgres_sql_lenient_with_sources(&sql, &sql);
    let before: Vec<_> = statements
        .iter()
        .map(|item| item.recovery_failure.clone())
        .collect();
    OriginalHeads::capture(&tokens).restore(&sql, &sql, &tokenized, &map, &mut statements);
    assert_eq!(
        statements
            .iter()
            .map(|item| item.recovery_failure.clone())
            .collect::<Vec<_>>(),
        before
    );
}
