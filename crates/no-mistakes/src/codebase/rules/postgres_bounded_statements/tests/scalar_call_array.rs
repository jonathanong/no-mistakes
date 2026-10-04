use super::unbounded;
#[test]
fn scalar_calls_preserve_finite_array_leaves_without_trusting_unknown_results() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/scalar-call-array.sql"
    ));
    for (index, line) in sql.lines().enumerate().skip(1) {
        assert!(
            !crate::codebase::postgres::extract_sql_statement_facts(line).parse_failed,
            "saved SQL line {}: {}",
            index + 1,
            line
        );
    }
    assert_eq!(
        unbounded(sql),
        [3, 4]
            .into_iter()
            .chain(6..=20)
            .chain(22..=27)
            .chain(30..=33)
            .chain(35..=40)
            .map(|line| ("accounts".to_string(), line))
            .collect::<Vec<_>>()
    );
}
