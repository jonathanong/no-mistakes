use super::facts;

#[test]
fn typed_builtin_unnest_outputs_keep_scalar_element_evidence() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/typed-unnest-outputs.sql"),
    )
    .unwrap();
    let found = facts(&sql);
    let scalars: Vec<_> = found
        .iter()
        .map(|fact| fact.query.outputs[0].scalar)
        .collect();
    assert_eq!(
        scalars,
        [
            true, true, true, false, false, false, false, false, false, false, false, true, false,
            false, false, false, false, true, false, false
        ],
    );
}
