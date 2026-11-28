use sqlparser::ast::Expr;

pub(super) fn visit_children(expr: &Expr, visit: &mut impl FnMut(&Expr)) {
    if visit_simple(expr, visit) {
        return;
    }
    visit_compound(expr, visit);
}

fn visit_simple(expr: &Expr, visit: &mut impl FnMut(&Expr)) -> bool {
    match expr {
        Expr::Nested(inner)
        | Expr::UnaryOp { expr: inner, .. }
        | Expr::Cast { expr: inner, .. }
        | Expr::IsNull(inner)
        | Expr::IsNotNull(inner)
        | Expr::IsTrue(inner)
        | Expr::IsNotTrue(inner)
        | Expr::IsFalse(inner)
        | Expr::IsNotFalse(inner)
        | Expr::IsUnknown(inner)
        | Expr::IsNotUnknown(inner)
        | Expr::Extract { expr: inner, .. }
        | Expr::Ceil { expr: inner, .. }
        | Expr::Floor { expr: inner, .. } => {
            visit(inner);
            true
        }
        Expr::BinaryOp { left, right, .. }
        | Expr::IsDistinctFrom(left, right)
        | Expr::IsNotDistinctFrom(left, right)
        | Expr::AnyOp { left, right, .. }
        | Expr::AllOp { left, right, .. }
        | Expr::Like {
            expr: left,
            pattern: right,
            ..
        }
        | Expr::ILike {
            expr: left,
            pattern: right,
            ..
        } => {
            visit(left);
            visit(right);
            true
        }
        _ => false,
    }
}

fn visit_compound(expr: &Expr, visit: &mut impl FnMut(&Expr)) {
    match expr {
        Expr::Case {
            operand,
            conditions,
            else_result,
            ..
        } => {
            operand.iter().for_each(|op| visit(op));
            for when in conditions {
                visit(&when.condition);
                visit(&when.result);
            }
            else_result.iter().for_each(|other| visit(other));
        }
        Expr::Between {
            expr, low, high, ..
        } => {
            visit(expr);
            visit(low);
            visit(high);
        }
        Expr::Tuple(items) => items.iter().for_each(visit),
        Expr::Array(array) => array.elem.iter().for_each(visit),
        _ => {}
    }
}
