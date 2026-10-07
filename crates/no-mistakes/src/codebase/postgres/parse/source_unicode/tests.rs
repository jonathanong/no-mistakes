use crate::codebase::postgres::parse::prepare_postgres_tokens;
use sqlparser::tokenizer::Token;

#[test]
fn same_inventory_preserves_postgres_operators_and_non_unicode_u_operands() {
    let sql = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/unicode-token-controls.sql"),
    )
    .unwrap();
    let prepared = prepare_postgres_tokens(&sql);
    assert!(prepared.lexical_error.is_none());
    let tokens = &prepared.tokens;
    assert!(tokens.iter().any(|t| t.token == Token::Question));
    assert!(tokens.iter().any(|t| t.token == Token::TwoWayArrow));
    assert_eq!(
        tokens.iter().filter(|t| matches!(&t.token, Token::Word(word) if word.quote_style.is_none() && word.value == "u")).count(),
        2
    );
    assert!(tokens.iter().any(|t| matches!(&t.token, Token::Word(word) if word.quote_style == Some('"') && word.value == "U")));
    let unicode: Vec<_> = tokens
        .iter()
        .filter_map(|t| match &t.token {
            Token::UnicodeStringLiteral(value) => Some(value.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(unicode, ["data"]);
    let report = crate::codebase::postgres::source::parse_postgres_source(
        &crate::codebase::postgres::source::PostgresSqlSource {
            sql,
            file_name: Some("unicode-token-controls.sql".into()),
        },
    );
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(report.statements.len(), 1);
}
