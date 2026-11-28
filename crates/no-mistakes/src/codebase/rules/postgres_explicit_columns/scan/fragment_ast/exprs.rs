use super::{FragmentStar, Rel};
use sqlparser::ast::{Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, InTarget};

pub(super) fn walk_subqueries(expr: &Expr, ctes: &[String], out: &mut Vec<FragmentStar>) {
    match expr {
        Expr::Exists { subquery, .. } => super::collect_query(subquery, ctes, true, out),
        Expr::Subquery(subquery) => super::collect_query(subquery, ctes, false, out),
        Expr::In { expr, target, .. } => {
            walk_subqueries(expr, ctes, out);
            walk_in_target(target, ctes, out, &mut |e, c, o| walk_subqueries(e, c, o));
        }
        Expr::Function(func) => walk_function_subqueries(func, ctes, out),
        _ => visit_children(expr, &mut |child| walk_subqueries(child, ctes, out)),
    }
}

pub(super) fn walk_expr_stars(
    expr: &Expr,
    rels: &[Rel],
    ctes: &[String],
    returning: bool,
    enclosing_fn: Option<&str>,
    line: usize,
    out: &mut Vec<FragmentStar>,
) {
    use sqlparser::ast::Spanned;
    let line = (expr.span().start.line as usize).max(line);
    match expr {
        Expr::Wildcard(_) if !enclosing_fn.is_some_and(is_count_fn) => {
            super::items::emit_bare(rels, returning, enclosing_fn.map(str::to_string), line, out);
        }
        Expr::QualifiedWildcard(q, ..) => {
            let func = enclosing_fn.map(str::to_string);
            super::items::emit_qualified(rels, q, returning, func, line, out);
        }
        Expr::Nested(inner) => {
            walk_expr_stars(inner, rels, ctes, returning, enclosing_fn, line, out);
        }
        Expr::Function(func) => walk_function_stars(func, rels, ctes, returning, line, out),
        Expr::Exists { subquery, .. } => super::collect_query(subquery, ctes, true, out),
        Expr::Subquery(subquery) => super::collect_query(subquery, ctes, false, out),
        Expr::In { expr, target, .. } => {
            walk_expr_stars(expr, rels, ctes, returning, None, line, out);
            walk_in_target(target, ctes, out, &mut |e, c, o| {
                walk_expr_stars(e, rels, c, returning, None, line, o);
            });
        }
        _ => visit_children(expr, &mut |child| {
            walk_expr_stars(child, rels, ctes, returning, None, line, out);
        }),
    }
}

fn walk_in_target(
    target: &InTarget,
    ctes: &[String],
    out: &mut Vec<FragmentStar>,
    walk_expr: &mut impl FnMut(&Expr, &[String], &mut Vec<FragmentStar>),
) {
    match target {
        InTarget::Subquery(subquery) => super::collect_query(subquery, ctes, false, out),
        InTarget::List(list) => list.iter().for_each(|i| walk_expr(i, ctes, out)),
        InTarget::Unnest(inner) => walk_expr(inner, ctes, out),
        InTarget::Table { .. } => {}
    }
}

fn walk_function_subqueries(func: &Function, ctes: &[String], out: &mut Vec<FragmentStar>) {
    match &func.args {
        FunctionArguments::Subquery(subquery) => super::collect_query(subquery, ctes, false, out),
        FunctionArguments::List(args) => {
            for arg in &args.args {
                if let FunctionArgExpr::Expr(e) = arg_expr(arg) {
                    walk_subqueries(e, ctes, out);
                }
            }
        }
        FunctionArguments::None => {}
    }
}

fn walk_function_stars(
    func: &Function,
    rels: &[Rel],
    ctes: &[String],
    returning: bool,
    line: usize,
    out: &mut Vec<FragmentStar>,
) {
    let name = no_mistakes_postgres_sql::idents::object_name_key(&func.name);
    match &func.args {
        FunctionArguments::Subquery(subquery) => super::collect_query(subquery, ctes, false, out),
        FunctionArguments::List(args) => {
            for arg in &args.args {
                match arg_expr(arg) {
                    FunctionArgExpr::Wildcard if !is_count_fn(&name) => {
                        super::items::emit_bare(rels, returning, Some(name.clone()), line, out);
                    }
                    FunctionArgExpr::QualifiedWildcard(q) => {
                        let func = Some(name.clone());
                        super::items::emit_qualified(rels, q, returning, func, line, out);
                    }
                    FunctionArgExpr::Expr(e) => {
                        walk_expr_stars(e, rels, ctes, returning, Some(&name), line, out);
                    }
                    _ => {}
                }
            }
        }
        FunctionArguments::None => {}
    }
}

fn is_count_fn(name: &str) -> bool {
    name.eq_ignore_ascii_case("count") || name.eq_ignore_ascii_case("pg_catalog.count")
}

pub(super) fn arg_expr(arg: &FunctionArg) -> &FunctionArgExpr {
    match arg {
        FunctionArg::Unnamed(a)
        | FunctionArg::Named { arg: a, .. }
        | FunctionArg::ExprNamed { arg: a, .. } => a,
    }
}

fn visit_children(expr: &Expr, visit: &mut impl FnMut(&Expr)) {
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
