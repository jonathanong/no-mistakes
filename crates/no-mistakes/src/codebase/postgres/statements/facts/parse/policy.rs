use super::*;

pub(crate) fn extract_sql_statement_facts_with_policy(
    sql: &str,
    collect_bounds: bool,
    positions: PlaceholderPositions<'_>,
    demand: SqlLocationDemand,
    policy: super::super::StatementPolicySources<'_>,
    lock_scope: bool,
) -> SqlStatementFileFacts {
    let projection = match (policy.collect_body_bounds, lock_scope) {
        (true, true) => StatementProjection::BodyBoundsLockScope,
        (true, false) => StatementProjection::BodyBounds,
        (false, true) => StatementProjection::LockScope,
        (false, false) => StatementProjection::Complete,
    };
    extract_sql_statement_facts_with_placeholder_positions(
        sql,
        collect_bounds,
        positions,
        demand,
        projection,
        policy,
    )
}
