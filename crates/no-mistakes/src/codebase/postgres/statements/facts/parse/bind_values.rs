use super::{prepared, StatementProjection};
use crate::codebase::postgres::statement_facts::SqlLocationDemand;
use crate::codebase::postgres::SqlStatementFileFacts;

pub(crate) fn extract_with_bound_values(
    sql: &str,
    positions: &[(u32, u32)],
    demand: SqlLocationDemand,
    locks: bool,
    policy: super::super::StatementPolicySources<'_>,
) -> SqlStatementFileFacts {
    let projection = match (policy.collect_body_bounds, locks) {
        (true, true) => StatementProjection::BodyBoundsLockScope,
        (true, false) => StatementProjection::BodyBounds,
        (false, true) => StatementProjection::LockScope,
        (false, false) => StatementProjection::Complete,
    };
    prepared::extract_with_bind_values(sql, true, Some(positions), demand, projection, policy)
}
