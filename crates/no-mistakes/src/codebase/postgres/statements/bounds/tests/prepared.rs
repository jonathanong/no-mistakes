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
        15
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
            "select: opaque"
        ]
    );
}
