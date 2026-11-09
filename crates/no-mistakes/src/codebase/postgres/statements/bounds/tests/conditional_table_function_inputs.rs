use crate::codebase::postgres::SqlBoundItemKind;

#[test]
fn caller_conditional_inputs_preserve_from_and_projection_cardinality() {
    let facts = super::facts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/conditional-table-function-inputs.sql"
    )));
    assert_eq!(facts.len(), 22);
    let owning: Vec<_> = facts
        .iter()
        .filter(|fact| fact.statement_start == Some((fact.line, fact.column)))
        .collect();
    assert_eq!(owning.len(), 21);
    for (index, fact) in owning.iter().enumerate() {
        let opaque = fact
            .query
            .items
            .iter()
            .any(|item| matches!(item.kind, SqlBoundItemKind::Opaque));
        assert_eq!(
            opaque,
            [5, 6, 7, 8, 9, 10, 11, 14, 15, 17, 18, 19, 20].contains(&index),
            "index {index}: {fact:?}"
        );
    }
    let scalars: Vec<_> = facts
        .iter()
        .filter(|fact| fact.statement_start != Some((fact.line, fact.column)))
        .collect();
    assert_eq!(scalars.len(), 1);
    let scalar = scalars[0];
    assert_eq!(
        scalar.kind,
        crate::codebase::postgres::statements::SqlBoundKind::Update
    );
    assert_eq!(scalar.line, 11);
    assert!(scalar.target.is_none() && scalar.body.is_none());
    assert!(matches!(
        &scalar.query.items[0].kind,
        SqlBoundItemKind::Table(name) if name == "orders"
    ));
}
