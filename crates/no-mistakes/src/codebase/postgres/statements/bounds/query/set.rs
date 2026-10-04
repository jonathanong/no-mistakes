use super::super::{aggregate, table};
use super::{items, sized_by_itself, start, Scope};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlBoundQuery};
use sqlparser::ast::{SetExpr, Spanned};

pub(super) fn bound(
    set: &SetExpr,
    scope: &Scope,
    positions: super::super::super::value::PlaceholderPositions<'_>,
) -> SqlBoundQuery {
    match set {
        SetExpr::Select(select) => SqlBoundQuery {
            capped: aggregate::pure_aggregate(select)
                || super::super::predicate::rejects_all(select.selection.as_ref()),
            items: items::from_select(select, scope, positions),
            outputs: super::outputs::select(select, positions),
        },
        SetExpr::Query(query) => super::bound_query(query, scope, positions),
        // A set operation returns the rows of both arms, so both must be bounded.
        SetExpr::SetOperation { left, right, .. } => {
            let left_bound = bound(left, scope, positions);
            scope.advance_table_tokens_to_right_arm(left.span().start);
            let right_bound = bound(right, scope, positions);
            let outputs = super::outputs::common(&left_bound.outputs, &right_bound.outputs);
            let items = vec![arm(left_bound, left), arm(right_bound, right)];
            SqlBoundQuery {
                capped: false,
                items,
                outputs,
            }
        }
        SetExpr::Table(table) => table::bound(table, scope, start(set.span())),
        _ => sized_by_itself(start(set.span())),
    }
}

fn arm(bound: SqlBoundQuery, set: &SetExpr) -> SqlBoundItem {
    SqlBoundItem::new(SqlBoundItemKind::Query(bound), None, start(set.span()))
}
