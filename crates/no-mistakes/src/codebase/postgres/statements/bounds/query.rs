use super::aggregate::orders_by_aggregate;
use super::{items, start, Scope};
use crate::codebase::postgres::statements::limit::is_zero_limited;
use crate::codebase::postgres::statements::{SqlBoundItemKind, SqlBoundQuery};
use sqlparser::ast::{Query, SetExpr, SetOperator, SetQuantifier, Statement};
mod blocking;
mod compact;
mod recursive_order;
mod set;
mod with_scope;
pub(super) use with_scope::with_scope;
#[cfg(test)]
mod tests;
use blocking::BlockingStatuses;

pub(super) fn bound_query(
    query: &Query,
    scope: &Scope,
    positions: super::super::value::PlaceholderPositions<'_>,
) -> SqlBoundQuery {
    bound_body(query, &with_scope(query, scope, positions), positions)
}

/// The `INSERT` / `UPDATE` / `DELETE` / `MERGE` inside a data-modifying CTE.
pub(super) fn modifying_statement(query: &Query) -> Option<&Statement> {
    match &*query.body {
        SetExpr::Update(statement)
        | SetExpr::Delete(statement)
        | SetExpr::Insert(statement)
        | SetExpr::Merge(statement) => Some(statement),
        _ => None,
    }
}

/// The body of `query` under `scope`, which already holds its CTEs.
pub(super) fn bound_body(
    query: &Query,
    scope: &Scope,
    positions: super::super::value::PlaceholderPositions<'_>,
) -> SqlBoundQuery {
    bound_body_observed(query, scope, positions, |_| {})
}

fn bound_body_observed(
    query: &Query,
    scope: &Scope,
    positions: super::super::value::PlaceholderPositions<'_>,
    observe: impl FnOnce(&BlockingStatuses),
) -> SqlBoundQuery {
    let mut bound = set::bound(&query.body, scope, positions);
    if let Some(capped) = super::aggregate::order_expansion_predicates_reject(query) {
        bound.capped = capped;
    }
    if is_zero_limited(query) {
        bound.capped = true;
    } else if super::super::limit::is_limited_at(query, positions) {
        if let Some(mut blocking) = blocking_statuses(&query.body) {
            cap_streaming_arms(&query.body, &mut bound, &mut blocking);
            observe(&blocking);
        } else {
            bound.capped = true;
        }
    }
    bound.capped |= orders_by_aggregate(query);
    bound
}

// A fully streaming query needs only its root cap and no status storage. Once a blocking
// subtree exists, the second linear walk records each relevant subtree exactly once.
fn blocking_statuses(set: &SetExpr) -> Option<BlockingStatuses> {
    if !contains_blocking(set) {
        return None;
    }
    let mut blocking = BlockingStatuses::new();
    requires_complete_arms(set, &mut blocking);
    Some(blocking)
}

fn contains_blocking(set: &SetExpr) -> bool {
    match set {
        SetExpr::SetOperation {
            op, set_quantifier, ..
        } if *op != SetOperator::Union || *set_quantifier != SetQuantifier::All => true,
        SetExpr::SetOperation { left, right, .. } => {
            contains_blocking(left) || contains_blocking(right)
        }
        SetExpr::Query(query) => contains_blocking(&query.body),
        _ => false,
    }
}

/// Non-streaming set operations must inspect their input arms before an outer LIMIT can
/// produce the final rows; UNION ALL can stop as soon as that limit is satisfied.
fn requires_complete_arms(set: &SetExpr, blocking: &mut BlockingStatuses) -> bool {
    blocking.visits += 1;
    let complete = match set {
        SetExpr::SetOperation {
            op,
            set_quantifier,
            left,
            right,
        } if *op == SetOperator::Union && *set_quantifier == SetQuantifier::All => {
            // Visit both arms even when the left one blocks, because capping later
            // needs each streaming sibling's status without another AST scan.
            let left_complete = requires_complete_arms(left, blocking);
            let right_complete = requires_complete_arms(right, blocking);
            left_complete || right_complete
        }
        SetExpr::SetOperation { .. } => true,
        SetExpr::Query(query) => requires_complete_arms(&query.body, blocking),
        _ => false,
    };
    blocking.by_set.insert(set as *const SetExpr, complete);
    complete
}

/// A UNION ALL limit caps each streaming sibling independently. A blocking subtree keeps
/// its input findings even when another sibling can stop after the requested rows.
fn cap_streaming_arms(set: &SetExpr, bound: &mut SqlBoundQuery, blocking: &mut BlockingStatuses) {
    if !blocking.complete(set) {
        bound.capped = true;
        return;
    }
    match set {
        SetExpr::SetOperation {
            op: SetOperator::Union,
            set_quantifier: SetQuantifier::All,
            left,
            right,
        } => {
            for (set, item) in [left, right].into_iter().zip(&mut bound.items) {
                if let SqlBoundItemKind::Query(inner) = &mut item.kind {
                    cap_streaming_arms(set, inner, blocking);
                }
            }
        }
        SetExpr::Query(query) => cap_streaming_arms(&query.body, bound, blocking),
        _ => {}
    }
}

/// A body whose size nothing in the statement text decides: it adds no unbounded relation.
pub(super) fn sized_by_itself(at: (usize, usize)) -> SqlBoundQuery {
    SqlBoundQuery {
        capped: false,
        items: vec![items::other(at)],
    }
}
