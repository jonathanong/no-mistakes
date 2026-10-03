use super::Tokens;
use sqlparser::tokenizer::{Location, Span};

const RADIX_SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-hex-literals/sql/pages.sql"
));

#[test]
fn source_lookup_uses_unicode_scalar_columns() {
    let line = RADIX_SQL.lines().next().unwrap();
    let column = line.find("0xF_F").unwrap();
    let column = line[..column].chars().count() as u64 + 1;
    let tokens = Tokens::new(RADIX_SQL);

    assert_eq!(
        tokens.source_at(Span::new(
            Location::new(1, column),
            Location::new(1, column + "0xF_F".len() as u64),
        )),
        Some("0xF_F")
    );
}

#[test]
fn source_lookup_rejects_unknown_or_out_of_range_positions() {
    let tokens = Tokens::new(RADIX_SQL);
    assert_eq!(tokens.source_at(Span::empty()), None);
    assert_eq!(
        tokens.source_at(Span::new(Location::new(100, 1), Location::new(100, 2))),
        None
    );
}

#[test]
fn source_lookup_rejects_unavailable_end_and_reversed_spans() {
    let tokens = Tokens::new(RADIX_SQL);
    assert_eq!(
        tokens.source_at(Span::new(Location::new(1, 1), Location::new(100, 1))),
        None
    );
    assert_eq!(
        tokens.source_at(Span::new(Location::new(1, 8), Location::new(1, 1))),
        None
    );
}

#[test]
fn source_lookup_can_reach_the_saved_source_end() {
    let tokens = Tokens::new(RADIX_SQL);
    let end_line = RADIX_SQL.lines().count() as u64 + 1;
    assert_eq!(
        tokens.source_at(Span::new(
            Location::new(end_line, 1),
            Location::new(end_line, 1)
        )),
        Some("")
    );
}

#[test]
fn indexed_source_preserves_many_same_line_and_multiline_hex_caps() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-hex-literals/sql/many.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert_eq!(facts.limit_uses.len(), 200);
    for (index, cap) in facts.limit_uses.iter().enumerate() {
        assert_eq!(
            cap.value,
            crate::codebase::postgres::SqlLimitValue::Literal(index as u64 + 1)
        );
        assert_eq!(cap.line, if index < 100 { 1 } else { index - 98 });
    }
}

#[test]
fn source_lookup_rejects_zero_column_with_a_valid_line() {
    let tokens = Tokens::new(RADIX_SQL);
    assert_eq!(
        tokens.source_at(Span::new(Location::new(1, 0), Location::new(1, 1))),
        None
    );
}

#[test]
fn source_lookup_rejects_columns_beyond_the_saved_line() {
    let tokens = Tokens::new(RADIX_SQL);
    assert_eq!(
        tokens.source_at(Span::new(
            Location::new(1, u64::MAX),
            Location::new(1, u64::MAX)
        )),
        None
    );
}
