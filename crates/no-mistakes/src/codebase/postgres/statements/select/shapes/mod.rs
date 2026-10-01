mod line;

#[cfg(test)]
mod tests;

use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::schema::relation_name;
use sqlparser::ast::{
    BinaryOperator, Expr, FunctionArguments, GroupByExpr, Query, Select, SelectItem, SetExpr,
    UnaryOperator, Value,
};

pub(super) struct ShapeLines {
    pub not_in_subqueries: Vec<usize>,
    pub count_existence_checks: Vec<usize>,
}

pub(super) fn collect(sql: &str, select: &Select) -> ShapeLines {
    let mut out = ShapeLines {
        not_in_subqueries: Vec::new(),
        count_existence_checks: Vec::new(),
    };
    let mut cursor = 0usize;
    let bare = !has_group_by(select);
    for item in &select.projection {
        if let SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } = item {
            walk(sql, expr, Place::SelectList, bare, &mut cursor, &mut out);
        }
    }
    for table in &select.from {
        for join in &table.joins {
            if let Some(expr) = super::from::join_expr(&join.join_operator) {
                walk(sql, expr, Place::Join, false, &mut cursor, &mut out);
            }
        }
    }
    if let Some(selection) = &select.selection {
        walk(sql, selection, Place::Where, bare, &mut cursor, &mut out);
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Place {
    SelectList,
    Where,
    Join,
}

fn walk(
    sql: &str,
    expr: &Expr,
    place: Place,
    bare: bool,
    cursor: &mut usize,
    out: &mut ShapeLines,
) {
    let expr = unwrap_expr(expr);
    match expr {
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            expr: inner,
        } => {
            if let Expr::InSubquery { negated: false, .. } = unwrap_expr(inner) {
                out.not_in_subqueries.push(line::line_of(sql, cursor, expr));
            }
            walk(sql, inner, place, bare, cursor, out);
        }
        Expr::InSubquery {
            negated,
            expr: left,
            ..
        } => {
            if *negated {
                out.not_in_subqueries.push(line::line_of(sql, cursor, expr));
            }
            walk(sql, left, place, bare, cursor, out);
        }
        Expr::BinaryOp { left, op, right } => {
            if existence(op, left, right, bare_allowed(place, bare)) {
                out.count_existence_checks
                    .push(line::line_of(sql, cursor, expr));
            }
            walk(sql, left, place, bare, cursor, out);
            walk(sql, right, place, bare, cursor, out);
        }
        other => crate::codebase::postgres::idents::visit_child_exprs(other, &mut |child| {
            walk(sql, child, place, bare, cursor, out);
        }),
    }
}

fn bare_allowed(place: Place, bare: bool) -> bool {
    bare && matches!(place, Place::SelectList | Place::Where)
}

fn existence(op: &BinaryOperator, left: &Expr, right: &Expr, bare: bool) -> bool {
    if is_count_side(left, bare) {
        int_literal(right).is_some_and(|number| pair(op, true, number))
    } else if is_count_side(right, bare) {
        int_literal(left).is_some_and(|number| pair(op, false, number))
    } else {
        false
    }
}

fn pair(op: &BinaryOperator, literal_on_right: bool, number: u8) -> bool {
    matches!(
        (op, literal_on_right, number),
        (&BinaryOperator::Gt, true, 0)
            | (&BinaryOperator::Lt, false, 0)
            | (&BinaryOperator::GtEq, true, 1)
            | (&BinaryOperator::LtEq, false, 1)
            | (&BinaryOperator::NotEq, _, 0)
            | (&BinaryOperator::Eq, _, 0)
            | (&BinaryOperator::Lt, true, 1)
            | (&BinaryOperator::Gt, false, 1)
            | (&BinaryOperator::LtEq, true, 0)
            | (&BinaryOperator::GtEq, false, 0)
    )
}

fn is_count_side(expr: &Expr, bare: bool) -> bool {
    match unwrap_expr(expr) {
        Expr::Subquery(query) => count_query(query),
        Expr::Function(_) if bare => is_count_function(expr),
        _ => false,
    }
}

fn count_query(query: &Query) -> bool {
    let SetExpr::Select(select) = query.body.as_ref() else {
        return false;
    };
    !has_group_by(select) && projection_is_count(select)
}

fn projection_is_count(select: &Select) -> bool {
    let [item] = select.projection.as_slice() else {
        return false;
    };
    match item {
        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
            is_count_function(expr)
        }
        _ => false,
    }
}

fn is_count_function(expr: &Expr) -> bool {
    let Expr::Function(function) = unwrap_expr(expr) else {
        return false;
    };
    if !relation_name(&function.name).eq_ignore_ascii_case("count") {
        return false;
    }
    matches!(function.args, FunctionArguments::List(_))
}

fn has_group_by(select: &Select) -> bool {
    match &select.group_by {
        GroupByExpr::Expressions(exprs, _) => !exprs.is_empty(),
        GroupByExpr::All(_) => true,
    }
}

fn int_literal(expr: &Expr) -> Option<u8> {
    let Expr::Value(value) = unwrap_expr(expr) else {
        return None;
    };
    let Value::Number(number, _) = &value.value else {
        return None;
    };
    match number.to_string().as_str() {
        "0" => Some(0),
        "1" => Some(1),
        _ => None,
    }
}
