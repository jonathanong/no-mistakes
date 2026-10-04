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
        },
        SetExpr::Query(query) => super::bound_query(query, scope, positions),
        // A set operation returns the rows of both arms, so both must be bounded.
        SetExpr::SetOperation { left, right, .. } => SqlBoundQuery {
            capped: false,
            items: {
                let left_bound = arm(left, scope, positions);
                // The left arm can contain TABLE syntax in projections that do not determine
                // its bound. Keep those tokens from lending a name to the right arm.
                scope.advance_table_tokens_to_right_arm(left.span().start);
                vec![left_bound, arm(right, scope, positions)]
            },
        },
        SetExpr::Table(table) => table::bound(table, scope, start(set.span())),
        _ => sized_by_itself(start(set.span())),
    }
}

fn arm(
    set: &SetExpr,
    scope: &Scope,
    positions: super::super::super::value::PlaceholderPositions<'_>,
) -> SqlBoundItem {
    let kind = SqlBoundItemKind::Query(bound(set, scope, positions));
    SqlBoundItem::new(kind, None, start(set.span()))
}
