use super::{FragmentStar, Rel};
use sqlparser::ast::{Expr, FunctionArg, FunctionArgExpr, FunctionArguments, InTarget};

pub(super) fn walk_subqueries(expr: &Expr, ctes: &[String], out: &mut Vec<FragmentStar>) {
    match expr {
        Expr::Exists { subquery, .. } => super::collect_query(subquery, ctes, true, out),
        Expr::Subquery(subquery) => super::collect_query(subquery, ctes, false, out),
        Expr::In { expr, target, .. } => {
            walk_subqueries(expr, ctes, out);
            match target {
                InTarget::Subquery(subquery) => super::collect_query(subquery, ctes, false, out),
                InTarget::List(list) => list.iter().for_each(|i| walk_subqueries(i, ctes, out)),
                InTarget::Unnest(inner) => walk_subqueries(inner, ctes, out),
                InTarget::Table { .. } => {}
            }
        }
        Expr::Function(func) => {
            if let FunctionArguments::Subquery(subquery) = &func.args {
                super::collect_query(subquery, ctes, false, out);
            } else {
                visit_children(expr, &mut |child| walk_subqueries(child, ctes, out));
            }
        }
        _ => visit_children(expr, &mut |child| walk_subqueries(child, ctes, out)),
    }
}

pub(super) fn walk_expr_stars(
    expr: &Expr,
    rels: &[Rel],
    ctes: &[String],
    returning: bool,
    line: usize,
    out: &mut Vec<FragmentStar>,
) {
    use sqlparser::ast::Spanned;
    let line = (expr.span().start.line as usize).max(line);
    match expr {
        Expr::Function(func) => {
            let name = no_mistakes_postgres_sql::idents::object_name_key(&func.name);
            match &func.args {
                FunctionArguments::Subquery(subquery) => {
                    super::collect_query(subquery, ctes, false, out);
                }
                FunctionArguments::List(args) => {
                    for arg in &args.args {
                        let inner = match arg {
                            FunctionArg::Unnamed(a)
                            | FunctionArg::Named { arg: a, .. }
                            | FunctionArg::ExprNamed { arg: a, .. } => a,
                        };
                        match inner {
                            FunctionArgExpr::Wildcard if !name.eq_ignore_ascii_case("count") => {
                                super::items::emit_bare(
                                    rels,
                                    returning,
                                    Some(name.clone()),
                                    line,
                                    out,
                                );
                            }
                            FunctionArgExpr::QualifiedWildcard(q) => {
                                super::items::emit_qualified(
                                    rels,
                                    q,
                                    returning,
                                    Some(name.clone()),
                                    line,
                                    out,
                                );
                            }
                            FunctionArgExpr::Expr(e) => {
                                walk_expr_stars(e, rels, ctes, returning, line, out);
                            }
                            _ => {}
                        }
                    }
                }
                FunctionArguments::None => {}
            }
        }
        Expr::Exists { subquery, .. } => super::collect_query(subquery, ctes, true, out),
        Expr::Subquery(subquery) => super::collect_query(subquery, ctes, false, out),
        Expr::In { expr, target, .. } => {
            walk_expr_stars(expr, rels, ctes, returning, line, out);
            match target {
                InTarget::Subquery(subquery) => super::collect_query(subquery, ctes, false, out),
                InTarget::List(list) => list
                    .iter()
                    .for_each(|i| walk_expr_stars(i, rels, ctes, returning, line, out)),
                InTarget::Unnest(inner) => walk_expr_stars(inner, rels, ctes, returning, line, out),
                InTarget::Table { .. } => {}
            }
        }
        _ => visit_children(expr, &mut |child| {
            walk_expr_stars(child, rels, ctes, returning, line, out);
        }),
    }
}

fn visit_children(expr: &Expr, visit: &mut impl FnMut(&Expr)) {
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
        | Expr::Floor { expr: inner, .. } => visit(inner),
        Expr::BinaryOp { left, right, .. }
        | Expr::AnyOp { left, right, .. }
        | Expr::AllOp { left, right, .. } => {
            visit(left);
            visit(right);
        }
        Expr::Case {
            operand,
            conditions,
            else_result,
            ..
        } => {
            if let Some(op) = operand {
                visit(op);
            }
            for when in conditions {
                visit(&when.condition);
                visit(&when.result);
            }
            if let Some(otherwise) = else_result {
                visit(otherwise);
            }
        }
        Expr::Between {
            expr, low, high, ..
        } => {
            visit(expr);
            visit(low);
            visit(high);
        }
        Expr::Like { expr, pattern, .. } | Expr::ILike { expr, pattern, .. } => {
            visit(expr);
            visit(pattern);
        }
        Expr::Tuple(items) => {
            for item in items {
                visit(item);
            }
        }
        Expr::Array(array) => {
            for item in &array.elem {
                visit(item);
            }
        }
        Expr::Function(func) => {
            if let FunctionArguments::List(args) = &func.args {
                for arg in &args.args {
                    let inner = match arg {
                        FunctionArg::Unnamed(a)
                        | FunctionArg::Named { arg: a, .. }
                        | FunctionArg::ExprNamed { arg: a, .. } => a,
                    };
                    if let FunctionArgExpr::Expr(e) = inner {
                        visit(e);
                    }
                }
            }
        }
        _ => {}
    }
}
