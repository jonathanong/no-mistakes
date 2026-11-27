use super::reject;
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::Tokenizer;

fn view() -> Statement {
    Parser::parse_sql(
        &PostgreSqlDialect {},
        "CREATE VIEW orders AS SELECT id FROM items",
    )
    .unwrap()
    .remove(0)
}

#[test]
fn missing_source_tokens_do_not_reject_a_view() {
    assert!(!reject(&view(), None));
}

#[test]
fn cascaded_check_without_option_rejects_the_view() {
    let tokens = Tokenizer::new(
        &PostgreSqlDialect {},
        "CREATE VIEW orders WITH CASCADED CHECK AS SELECT id FROM items",
    )
    .tokenize_with_location()
    .unwrap();
    assert!(reject(&view(), Some(&tokens)));
}
