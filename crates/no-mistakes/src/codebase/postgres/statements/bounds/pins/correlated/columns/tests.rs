#[test]
fn json_functions_preserve_declared_output_names_with_relation_aliases() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/json-function-output-columns.sql"));
    let statements =
        sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::PostgreSqlDialect {}, sql)
            .unwrap();
    assert_eq!(statements.len(), 11);
    for (index, statement) in statements.iter().enumerate() {
        let sqlparser::ast::Statement::Query(query) = statement else {
            panic!("saved fixture contains SELECT statements");
        };
        let reads =
            super::super::reads_outer_rows(query, &Default::default(), &Default::default(), None);
        assert_eq!(reads.certain, index == 9, "statement {index}");
        assert!(reads.bare.is_empty(), "statement {index}");
    }
}

#[test]
fn group_and_order_keywords_do_not_create_correlated_column_reads() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/derived-output-labels-duckdb.sql"
    ));
    let statements =
        sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::DuckDbDialect {}, sql).unwrap();
    let sqlparser::ast::Statement::Query(query) = &statements[0] else {
        panic!("saved fixture is a query");
    };
    let reads =
        super::super::reads_outer_rows(query, &Default::default(), &Default::default(), None);
    assert!(!reads.certain);
    assert!(reads.bare.is_empty());
}

#[test]
fn partial_table_aliases_preserve_only_known_projection_suffixes() {
    use sqlparser::ast::{SetExpr, Statement, TableFactor};

    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/partial-alias-projection-shapes.sql"
    ));
    let statements =
        sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::PostgreSqlDialect {}, sql)
            .unwrap();
    let expected = [
        Some(["id", "kind"].into_iter().map(str::to_string).collect()),
        Some(["id", "kind"].into_iter().map(str::to_string).collect()),
        Some(["id", "kind"].into_iter().map(str::to_string).collect()),
        None,
        None,
        None,
        None,
    ];
    assert_eq!(statements.len(), expected.len());
    for (index, (statement, expected)) in statements.iter().zip(expected).enumerate() {
        let Statement::Query(query) = statement else {
            panic!("saved fixture contains SELECT statements");
        };
        let SetExpr::Select(select) = &*query.body else {
            panic!("outer query is a SELECT");
        };
        let TableFactor::Derived {
            subquery,
            alias: Some(alias),
            ..
        } = &select.from[0].relation
        else {
            panic!("outer query has one aliased derived source");
        };
        assert_eq!(
            super::partial_alias_columns(subquery, alias),
            expected,
            "statement {index}"
        );
    }
}
