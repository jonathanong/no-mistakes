#[test]
fn radix_projection_values_are_repaired_before_implicit_alias_acceptance() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-prefix-separators/sql/projections.sql"));
    let statements = super::parse_postgres_sql(sql).unwrap();
    let printed: Vec<_> = statements.iter().map(ToString::to_string).collect();
    assert_eq!(
        printed,
        [
            "SELECT 15",
            "SELECT 2",
            "SELECT 15 + 1",
            "SELECT 0 AS \"o_17\"",
            "SELECT 0 AS o_17",
            "SELECT 0 AS \"o_17\""
        ]
    );
    let expr = crate::codebase::postgres::parse_postgres_expression(
        sql.lines()
            .next()
            .unwrap()
            .trim_start_matches("SELECT ")
            .trim_end_matches(';'),
    )
    .unwrap();
    assert_eq!(expr.to_string(), "15");
}
