#[test]
fn separators_respect_radix_prefix_and_digit_boundaries() {
    for (text, value) in [
        ("1_000", 1000),
        ("1_0_0", 100),
        ("5_0", 50),
        ("0xF_F", 255),
        ("0o7_0", 56),
        ("0b1_0", 2),
        ("0o_1_755", 1005),
        ("0b_1_0", 2),
        ("0x_F_F", 255),
    ] {
        assert_eq!(super::numeric_literal(text).unwrap(), value);
    }
    for text in ["1__0", "_1", "1_", "0x__F", "0x_", "0o__7", "0b1_2"] {
        assert!(super::numeric_literal(text).is_err(), "{text}");
    }
}
#[test]
fn numeric_hex_limits_keep_source_identity_and_column() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-hex-literals/sql/pages.sql"
    ));
    for (line, sql_line) in sql.lines().enumerate() {
        if sql_line.starts_with("SELECT") {
            assert!(
                crate::codebase::postgres::parse_postgres_sql(sql_line).is_ok(),
                "line {} did not parse",
                line + 1
            );
        }
    }
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);

    let expected = [
        (1, "0xF_F", super::SqlLimitValue::Literal(255)),
        (3, "0xF_F", super::SqlLimitValue::Literal(255)),
        (4, "X'FF'", super::SqlLimitValue::Other),
        (5, "x'FF'", super::SqlLimitValue::Other),
        (6, "0x0", super::SqlLimitValue::Literal(0)),
        (8, "0x1", super::SqlLimitValue::Literal(1)),
    ];
    assert_eq!(facts.limit_uses.len(), expected.len());
    for (fact, (line, literal, value)) in facts.limit_uses.iter().zip(expected) {
        let source_line = sql.lines().nth(line - 1).unwrap();
        let column = source_line.find(literal).unwrap();
        let column = source_line[..column].chars().count() + 1;
        assert_eq!((fact.line, fact.column, fact.value), (line, column, value));
    }
}
