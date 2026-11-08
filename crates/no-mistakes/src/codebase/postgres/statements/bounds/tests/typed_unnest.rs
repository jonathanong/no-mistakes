use crate::codebase::postgres::extract_sql_statement_facts;

#[test]
fn typed_builtin_unnest_outputs_keep_scalar_element_evidence() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/typed-unnest-outputs.sql"),
    )
    .unwrap();
    let scalar_lines = [1, 2, 3, 12, 18];
    assert_eq!(sql.lines().count(), 20);
    for (index, statement) in sql.lines().enumerate() {
        let line = index + 1;
        let found = extract_sql_statement_facts(statement);
        assert!(!found.parse_failed, "line {line}: {statement}: {found:?}");
        let outer = found
            .bounds
            .iter()
            .find(|fact| fact.line == 1 && fact.column == 1)
            .unwrap_or_else(|| panic!("line {line}: missing outer SELECT: {statement}"));
        assert_eq!(outer.query.outputs.len(), 1, "line {line}: {statement}");
        assert_eq!(
            outer.query.outputs[0].scalar,
            scalar_lines.contains(&line),
            "line {line}: {statement}",
        );
    }
}
