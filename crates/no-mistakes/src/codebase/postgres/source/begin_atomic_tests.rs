use super::{accept, parser};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::{Token, Tokenizer};

#[test]
fn accept_does_not_consume_a_non_atomic_word() {
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, "SELECT id FROM orders")
        .tokenize_with_location()
        .unwrap();
    let mut parsed = parser(tokens);
    assert!(!accept(&mut parsed));
    assert!(matches!(
        parsed.peek_token().token,
        Token::Word(word) if word.value == "SELECT"
    ));
}
