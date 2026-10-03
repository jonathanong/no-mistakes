#[test]
fn separators_require_digits_on_both_sides() {
    for (text, value) in [
        ("1_000", 1000),
        ("1_0_0", 100),
        ("5_0", 50),
        ("0xF_F", 255),
        ("0o7_0", 56),
        ("0b1_0", 2),
    ] {
        assert_eq!(super::numeric_literal(text).unwrap(), value);
    }
    for text in ["1__0", "_1", "1_", "0x_F", "0b1_2"] {
        assert!(super::numeric_literal(text).is_err(), "{text}");
    }
}

#[test]
fn semantic_zero_page_facts_preserve_literal_value_classification() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/semantic-zero/sql/pages.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    for line in sql.lines().filter(|line| line.starts_with("SELECT")) {
        assert!(
            sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::PostgreSqlDialect {}, line)
                .is_ok(),
            "{line}"
        );
    }
    assert!(!facts.parse_failed);
    assert_eq!(
        facts
            .sweeps
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        vec![23, 24, 25, 26, 28, 31]
    );
    assert!(facts
        .limit_uses
        .iter()
        .take(25)
        .enumerate()
        .all(|(index, fact)| fact.value
            == if index == 4 {
                super::SqlLimitValue::Literal(0)
            } else {
                super::SqlLimitValue::Other
            }));
    assert_eq!(facts.limit_uses[25].value, super::SqlLimitValue::Literal(0));
}
