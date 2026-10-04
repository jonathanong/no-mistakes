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

#[test]
fn catalog_numeric_zero_keeps_empty_page_proofs() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-sql-shape-policy/fixture/semantic-zero/sql/qualified-zero.sql"));
    let statements = crate::codebase::postgres::parse_postgres_sql(sql).unwrap();
    let empty: Vec<_> = statements
        .iter()
        .filter_map(|statement| match statement {
            sqlparser::ast::Statement::Query(query) => Some(super::is_empty_page(query)),
            _ => None,
        })
        .collect();
    assert_eq!(empty, [true, true, true, false, false, false, false]);
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(
        facts
            .sweeps
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [5, 6, 7, 9]
    );
}

#[test]
fn argumentless_function_ast_cannot_prove_a_fixed_count() {
    use sqlparser::ast::{Expr, FunctionArguments, LimitClause, Statement};
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/nullable-fixed-limit.sql"
    ));
    let mut statements = crate::codebase::postgres::parse_postgres_sql(sql).unwrap();
    let Statement::Query(query) = &mut statements[0] else {
        panic!("saved conditional-count fixture must start with SELECT");
    };
    let Some(LimitClause::LimitOffset {
        limit: Some(expr), ..
    }) = &mut query.limit_clause
    else {
        panic!("saved conditional-count fixture must have LIMIT");
    };
    let Expr::Function(function) = expr else {
        panic!("saved conditional-count fixture must have a function count");
    };
    // Prepared AST callers can supply this form even though PostgreSQL's parser writes a list.
    // Keep unsupported argument representations conservative at the shared fact boundary.
    function.args = FunctionArguments::None;
    assert!(!super::fixed_count::is_fixed_at(expr, None));
}
