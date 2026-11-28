use super::{extend_outer, FragmentStar, Rel, Scope};
use sqlparser::ast::{Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, InTarget};

mod children;
use children::visit_children;

pub(super) fn walk_subqueries(expr: &Expr, scope: Scope<'_>, out: &mut Vec<FragmentStar>) {
    match expr {
        Expr::Exists { subquery, .. } => super::collect_query(subquery, scope, true, out),
        Expr::Subquery(subquery) => super::collect_query(subquery, scope, false, out),
        Expr::In { expr, target, .. } => {
            walk_subqueries(expr, scope, out);
            walk_in_target(target, scope, out, &mut |e, s, o| walk_subqueries(e, s, o));
        }
        Expr::Function(func) => walk_function_subqueries(func, scope, out),
        _ => visit_children(expr, &mut |child| walk_subqueries(child, scope, out)),
    }
}

pub(super) fn walk_expr_stars(
    expr: &Expr,
    rels: &[Rel],
    scope: Scope<'_>,
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
            super::items::emit_qualified(rels, scope.outer_rels, q, returning, func, line, out);
        }
        Expr::Nested(inner) => {
            walk_expr_stars(inner, rels, scope, returning, enclosing_fn, line, out);
        }
        Expr::Function(func) => walk_function_stars(func, rels, scope, returning, line, out),
        Expr::Exists { subquery, .. } | Expr::Subquery(subquery) => {
            let outer = extend_outer(rels, scope.outer_rels);
            let child = Scope {
                ctes: scope.ctes,
                outer_rels: &outer,
            };
            let is_exists = matches!(expr, Expr::Exists { .. });
            super::collect_query(subquery, child, is_exists, out);
        }
        Expr::In { expr, target, .. } => {
            walk_expr_stars(expr, rels, scope, returning, None, line, out);
            let outer = extend_outer(rels, scope.outer_rels);
            let child = Scope {
                ctes: scope.ctes,
                outer_rels: &outer,
            };
            walk_in_target(target, child, out, &mut |e, _, o| {
                walk_expr_stars(e, rels, scope, returning, None, line, o);
            });
        }
        _ => visit_children(expr, &mut |child| {
            walk_expr_stars(child, rels, scope, returning, None, line, out);
        }),
    }
}

fn walk_in_target(
    target: &InTarget,
    scope: Scope<'_>,
    out: &mut Vec<FragmentStar>,
    walk_expr: &mut impl FnMut(&Expr, Scope<'_>, &mut Vec<FragmentStar>),
) {
    match target {
        InTarget::Subquery(subquery) => super::collect_query(subquery, scope, false, out),
        InTarget::List(list) => list.iter().for_each(|i| walk_expr(i, scope, out)),
        InTarget::Unnest(inner) => walk_expr(inner, scope, out),
        InTarget::Table { .. } => {}
    }
}

fn walk_function_subqueries(func: &Function, scope: Scope<'_>, out: &mut Vec<FragmentStar>) {
    match &func.args {
        FunctionArguments::Subquery(subquery) => super::collect_query(subquery, scope, false, out),
        FunctionArguments::List(args) => {
            for arg in &args.args {
                if let FunctionArgExpr::Expr(e) = arg_expr(arg) {
                    walk_subqueries(e, scope, out);
                }
            }
        }
        FunctionArguments::None => {}
    }
}

fn walk_function_stars(
    func: &Function,
    rels: &[Rel],
    scope: Scope<'_>,
    returning: bool,
    line: usize,
    out: &mut Vec<FragmentStar>,
) {
    let name = no_mistakes_postgres_sql::idents::object_name_key(&func.name);
    match &func.args {
        FunctionArguments::Subquery(subquery) => {
            let outer = extend_outer(rels, scope.outer_rels);
            let child = Scope {
                ctes: scope.ctes,
                outer_rels: &outer,
            };
            super::collect_query(subquery, child, false, out);
        }
        FunctionArguments::List(args) => {
            for arg in &args.args {
                match arg_expr(arg) {
                    FunctionArgExpr::Wildcard if !is_count_fn(&name) => {
                        super::items::emit_bare(rels, returning, Some(name.clone()), line, out);
                    }
                    FunctionArgExpr::QualifiedWildcard(q) => {
                        let f = Some(name.clone());
                        super::items::emit_qualified(
                            rels,
                            scope.outer_rels,
                            q,
                            returning,
                            f,
                            line,
                            out,
                        );
                    }
                    FunctionArgExpr::Expr(e) => {
                        walk_expr_stars(e, rels, scope, returning, Some(&name), line, out);
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
