mod identity;
pub(crate) use identity::{
    object_name_identity, parse_relation_identity, relation_part_key, relation_suffix_key,
    relation_suffix_name, resolve_relation_key,
};
use sqlparser::ast::Expr;
use std::collections::HashSet;

/// Lowercased identifiers referenced anywhere under `expr`.
pub fn collect_ident_names(expr: &Expr) -> Vec<String> {
    let mut names = Vec::new();
    collect_idents(expr, &mut names);
    names.sort();
    names.dedup();
    names
}

pub fn unwrap_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) => unwrap_expr(inner),
        other => other,
    }
}

pub(crate) fn visit_child_exprs(expr: &Expr, visit: &mut impl FnMut(&Expr)) {
    // Expression-named argument labels are syntax, not column references.
    if let Expr::Function(function) = expr {
        for args in [&function.parameters, &function.args] {
            if let sqlparser::ast::FunctionArguments::List(list) = args {
                visit_function_args(&list.args, visit);
            }
        }
        if let Some(filter) = &function.filter {
            visit(filter);
        }
        return;
    }
    use sqlparser::ast::{Visit, Visitor};
    use std::ops::ControlFlow;
    struct Children<'a, F> {
        visit: &'a mut F,
        depth: usize,
        queries: usize,
    }
    impl<F: FnMut(&Expr)> Visitor for Children<'_, F> {
        type Break = ();
        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            if self.depth == 1 && self.queries == 0 {
                (self.visit)(expr);
            }
            self.depth += 1;
            ControlFlow::Continue(())
        }
        fn post_visit_expr(&mut self, _: &Expr) -> ControlFlow<()> {
            self.depth -= 1;
            ControlFlow::Continue(())
        }
        fn pre_visit_query(&mut self, _: &sqlparser::ast::Query) -> ControlFlow<()> {
            self.queries += 1;
            ControlFlow::Continue(())
        }
        fn post_visit_query(&mut self, _: &sqlparser::ast::Query) -> ControlFlow<()> {
            self.queries -= 1;
            ControlFlow::Continue(())
        }
    }
    let _ = expr.visit(&mut Children {
        visit,
        depth: 0,
        queries: 0,
    });
}

pub(crate) fn object_name_key(name: &sqlparser::ast::ObjectName) -> String {
    name.0
        .iter()
        .filter_map(|part| match part {
            sqlparser::ast::ObjectNamePart::Identifier(ident) => Some(ident_key(ident)),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(".")
}

pub(crate) fn ident_key(ident: &sqlparser::ast::Ident) -> String {
    if ident.quote_style.is_some() {
        ident.value.clone()
    } else {
        ident.value.to_ascii_lowercase()
    }
}

pub(crate) fn object_name_ident(
    name: &sqlparser::ast::ObjectName,
) -> Option<&sqlparser::ast::Ident> {
    name.0.iter().rev().find_map(|part| match part {
        sqlparser::ast::ObjectNamePart::Identifier(ident) => Some(ident),
        _ => None,
    })
}

pub(crate) fn insert_ident(local: &mut HashSet<String>, ident: &sqlparser::ast::Ident) {
    let key = ident_key(ident);
    if !key.is_empty() {
        local.insert(key);
    }
}

fn collect_idents(expr: &Expr, names: &mut Vec<String>) {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => names.push(ident.value.to_ascii_lowercase()),
        Expr::CompoundIdentifier(parts) => {
            if let Some(ident) = parts.last() {
                names.push(ident.value.to_ascii_lowercase());
            }
        }
        other => visit_child_exprs(other, &mut |child| collect_idents(child, names)),
    }
}

pub(crate) fn visit_function_args(
    args: &[sqlparser::ast::FunctionArg],
    visit: &mut impl FnMut(&Expr),
) {
    for arg in args {
        if let Some(expr) = function_arg_expr(arg) {
            visit(expr);
        }
    }
}

fn function_arg_expr(arg: &sqlparser::ast::FunctionArg) -> Option<&Expr> {
    match arg {
        sqlparser::ast::FunctionArg::Unnamed(sqlparser::ast::FunctionArgExpr::Expr(expr))
        | sqlparser::ast::FunctionArg::Named {
            arg: sqlparser::ast::FunctionArgExpr::Expr(expr),
            ..
        }
        | sqlparser::ast::FunctionArg::ExprNamed {
            arg: sqlparser::ast::FunctionArgExpr::Expr(expr),
            ..
        } => Some(expr),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
