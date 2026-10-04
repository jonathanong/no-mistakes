use super::TableTokenCursor;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::{Token, Tokenizer};

#[test]
fn long_union_advances_through_each_operator_once() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/table-long-union.sql"),
    )
    .unwrap();
    let dialect = PostgreSqlDialect {};
    let tokens = Tokenizer::new(&dialect, &sql)
        .tokenize_with_location()
        .unwrap();
    let start = tokens
        .iter()
        .find(|token| matches!(&token.token, Token::Word(word) if word.value == "SELECT"))
        .unwrap()
        .span
        .start;
    let mut cursor = TableTokenCursor::new(&tokens);
    let operators: Vec<_> = cursor
        .operators
        .iter()
        .map(|operator| operator.at)
        .collect();
    assert_eq!(operators.len(), 64);
    for operator in operators {
        cursor.advance_to_right_arm(start);
        assert_eq!(cursor.last_operator, Some(operator));
    }
}

#[test]
fn long_union_advances_each_outer_from_past_nested_sources() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/table-long-from.sql"),
    )
    .unwrap();
    assert!(crate::codebase::postgres::parse_postgres_sql(&sql).is_ok());
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, &sql)
        .tokenize_with_location()
        .unwrap();
    let mut last_line = 0;
    let starts: Vec<_> = tokens
        .iter()
        .filter_map(|token| {
            if matches!(&token.token, Token::Word(word) if word.value == "SELECT")
                && token.span.start.line != last_line
            {
                last_line = token.span.start.line;
                Some(token.span.start)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(starts.len(), 64);
    let mut cursor = TableTokenCursor::new(&tokens);
    assert_eq!(cursor.froms.len(), 128);
    assert_eq!(cursor.names.len(), 128);
    for (arm, start) in starts.into_iter().enumerate() {
        cursor.advance_to_from(start);
        assert_eq!(cursor.next, arm * 2 + 1);
        assert_eq!(cursor.names[cursor.next].name, "public.accounts");
        cursor.next += 1;
    }
}
