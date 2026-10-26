use sqlparser::{dialect::PostgreSqlDialect, tokenizer::Tokenizer};

#[test]
fn invalid_if_label_stays_on_its_opener_but_loop_label_reaches_the_action() {
    let source = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-ddl-unanalyzable")
            .join("issue-2731/head-cases/label-heads.sql"),
    )
    .unwrap();
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, &source)
        .tokenize_with_location()
        .unwrap();
    let heads: Vec<_> = tokens
        .split(|token| token.token == sqlparser::tokenizer::Token::SemiColon)
        .filter_map(super::head)
        .collect();
    assert_eq!(heads.len(), 2);
    assert_eq!(heads[0].0.line, 2);
    assert!(heads[0].1.starts_with("<< condition >> IF"));
    assert_eq!(heads[1].0.line, 6);
    assert!(heads[1].1.starts_with("CREATE RECURSIVE VIEW valid_label"));
}

#[test]
fn incomplete_labels_and_conditions_keep_their_opener() {
    let source = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-ddl-unanalyzable")
            .join("issue-2731/head-cases/malformed-openers.sql"),
    )
    .unwrap();
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, &source)
        .tokenize_with_location()
        .unwrap();
    let heads: Vec<_> = tokens
        .split(|token| token.token == sqlparser::tokenizer::Token::SemiColon)
        .filter_map(super::head)
        .collect();
    assert_eq!(
        heads.iter().map(|head| head.0.line).collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert!(heads[0].1.starts_with("<<"));
    assert!(heads[1].1.starts_with("<< label"));
    assert!(heads[2].1.starts_with("IF true"));
}
