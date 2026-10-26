use super::*;

#[test]
fn policy_projection_excludes_plan_only_calls_without_changing_syntactic_calls() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres/function-call-clauses/explain.sql"
    ));
    let statements = crate::codebase::postgres::parse::parse_postgres_sql(sql).unwrap();
    let mut syntactic = Vec::new();
    let mut executed = Vec::new();
    for statement in &statements {
        collect(statement, &mut syntactic);
        collect_executed(statement, &mut executed);
    }
    assert_eq!(syntactic.len(), 6);
    assert_eq!(executed.len(), 3);
    assert_eq!(
        executed.iter().map(|call| call.line).collect::<Vec<_>>(),
        [5, 6, 7]
    );
}
