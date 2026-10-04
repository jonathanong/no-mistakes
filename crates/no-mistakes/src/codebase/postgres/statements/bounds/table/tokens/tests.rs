use super::{first_after, TableTokenCursor};
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

#[test]
fn long_chain_marker_lookups_have_logarithmic_comparison_counts() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/table-long-from.sql"),
    )
    .unwrap();
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, &sql)
        .tokenize_with_location()
        .unwrap();
    let cursor = TableTokenCursor::new(&tokens);

    // These are the same marker arrays and lookup helper used by arm advancement.
    // Count only lookup comparisons, not one-time tokenization or index construction.
    fn assert_bounded_search<T>(items: &[T], at: impl Fn(&T) -> (usize, usize)) {
        assert!(items.len() >= 63);
        // slice::partition_point may probe one extra element at its boundary.
        let budget = (usize::BITS - items.len().leading_zeros()) as usize + 1;
        for (index, item) in items.iter().enumerate() {
            let mut comparisons = 0;
            let found = first_after(items, at(item), |candidate| {
                comparisons += 1;
                at(candidate)
            });
            assert_eq!(found, index + 1);
            assert!(
                comparisons <= budget,
                "lookup inspected {comparisons} of {} markers; budget {budget}",
                items.len()
            );
        }
    }

    assert_bounded_search(&cursor.depths, |marker| marker.at);
    assert_bounded_search(&cursor.operators, |marker| marker.at);
    assert_bounded_search(&cursor.froms, |marker| marker.at);
}
