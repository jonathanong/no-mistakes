mod existence;
mod line;
use existence::existence;

#[cfg(test)]
mod tests;

use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::SqlCountExistenceFact;
use sqlparser::ast::{Expr, GroupByExpr, Select, SelectItem, UnaryOperator};

pub(super) struct ShapeLines {
    pub not_in_subqueries: Vec<usize>,
    pub not_in_columns: Vec<usize>,
    pub count_existence_checks: Vec<SqlCountExistenceFact>,
}

pub(super) fn collect(select: &Select) -> ShapeLines {
    let mut out = ShapeLines {
        not_in_subqueries: Vec::new(),
        not_in_columns: Vec::new(),
        count_existence_checks: Vec::new(),
    };
    let bare = !has_group_by(select);
    for item in &select.projection {
        if let SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } = item {
            walk(expr, Place::SelectList, bare, &mut out);
        }
    }
    for table in &select.from {
        for join in &table.joins {
            if let Some(expr) = super::from::join_expr(&join.join_operator) {
                walk(expr, Place::Join, false, &mut out);
            }
        }
    }
    if let Some(selection) = &select.selection {
        walk(selection, Place::Where, bare, &mut out);
    }
    if let Some(having) = &select.having {
        walk(having, Place::Having, bare, &mut out);
    }
    out
}

pub(super) fn collect_predicate(expr: &Expr) -> ShapeLines {
    let mut out = ShapeLines {
        not_in_subqueries: Vec::new(),
        not_in_columns: Vec::new(),
        count_existence_checks: Vec::new(),
    };
    walk(expr, Place::Where, false, &mut out);
    out
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Place {
    SelectList,
    Where,
    Join,
    Having,
}

fn walk(expr: &Expr, place: Place, bare: bool, out: &mut ShapeLines) {
    let expr = unwrap_expr(expr);
    match expr {
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            ..
        }
        | Expr::InSubquery { .. } => {
            let mut inner = expr;
            let mut negated = false;
            while let Expr::UnaryOp {
                op: UnaryOperator::Not,
                expr,
            } = unwrap_expr(inner)
            {
                negated = !negated;
                inner = expr;
            }
            if let Expr::InSubquery {
                negated: membership_negated,
                expr: left,
                ..
            } = unwrap_expr(inner)
            {
                if negated != *membership_negated {
                    out.not_in_subqueries.push(line::line_of(expr));
                    out.not_in_columns.push(line::column_of(expr));
                }
                walk(left, place, bare, out);
            } else if let Expr::BinaryOp { left, op, right } = unwrap_expr(inner) {
                if let Some(absence) = existence(op, left, right, bare_allowed(place, bare)) {
                    out.count_existence_checks.push(SqlCountExistenceFact {
                        line: line::line_of(expr),
                        column: line::column_of(expr),
                        negated: absence ^ negated,
                    });
                    walk(left, place, bare, out);
                    walk(right, place, bare, out);
                } else {
                    walk(inner, place, bare, out);
                }
            } else {
                walk(inner, place, bare, out);
            }
        }
        Expr::BinaryOp { left, op, right } => {
            if let Some(negated) = existence(op, left, right, bare_allowed(place, bare)) {
                out.count_existence_checks.push(SqlCountExistenceFact {
                    line: line::line_of(expr),
                    column: line::column_of(expr),
                    negated,
                });
            }
            walk(left, place, bare, out);
            walk(right, place, bare, out);
        }
        other => crate::codebase::postgres::idents::visit_child_exprs(other, &mut |child| {
            walk(child, place, bare, out);
        }),
    }
}

fn bare_allowed(place: Place, bare: bool) -> bool {
    bare && matches!(place, Place::SelectList | Place::Where | Place::Having)
}

fn has_group_by(select: &Select) -> bool {
    match &select.group_by {
        GroupByExpr::Expressions(exprs, _) => exprs.iter().any(|expr| match expr {
            Expr::Tuple(items) => !items.is_empty(),
            Expr::GroupingSets(sets) | Expr::Rollup(sets) | Expr::Cube(sets) => {
                sets.iter().any(|set| !set.is_empty())
            }
            _ => true,
        }),
        GroupByExpr::All(_) => true,
    }
}
