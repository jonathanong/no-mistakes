use super::shape;

#[test]
fn prepared_select_into_creates_temporary_identity_only_on_execute() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-prepared-into.sql"
    ));
    // Keep the saved PREPARE/EXECUTE sequence strict so recovery cannot silently skip a step.
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        20
    );
    assert_eq!(
        shape(sql),
        [
            "select: capped orders",
            "select: accounts",
            "select: opaque",
            "select: accounts",
            "select: capped orders",
            "select: accounts",
            "select: capped orders",
            "select: capped orders",
            "select: opaque",
            "select: capped orders",
            "select: accounts"
        ]
    );
}

#[test]
fn duplicate_definitions_and_invalid_execute_arity_preserve_temporary_identity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-prepared-validation.sql"
    ));
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        27
    );
    assert_eq!(
        shape(sql),
        [
            "select: capped orders",
            "select: capped orders",
            "select: accounts",
            "select: capped orders",
            "select: capped orders",
            "select: opaque",
            "select: capped orders[id=value]",
            "select: accounts",
            "select: opaque",
            "select: capped orders[id=value]",
            "select: accounts",
            "select: opaque",
            "select: capped orders[id=value]",
            "select: accounts"
        ]
    );
}

#[test]
fn partial_prepared_types_infer_total_execute_arity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-prepared-partial-types.sql"
    ));
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        19
    );
    assert_eq!(
        shape(sql),
        [
            "select: capped orders",
            "select: accounts",
            "select: accounts",
            "select: opaque",
            "select: capped orders",
            "select: accounts",
            "select: opaque",
            "select: capped orders",
            "select: accounts"
        ]
    );
}
