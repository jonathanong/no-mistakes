use sqlparser::ast::*;
use std::ops::ControlFlow;
pub(super) fn bare(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) => bare(inner),
        _ => expr,
    }
}
pub(super) fn column(expr: &Expr) -> Option<(Option<String>, String)> {
    match bare(expr) {
        Expr::Identifier(id) => Some((None, ident(id))),
        Expr::CompoundIdentifier(ids) if ids.len() == 2 => {
            Some((Some(ident(&ids[0])), ident(&ids[1])))
        }
        _ => None,
    }
}
pub(super) fn ident(id: &Ident) -> String {
    if id.quote_style.is_some() {
        id.value.clone()
    } else {
        id.value.to_ascii_lowercase()
    }
}
pub(super) fn conjuncts(expr: Option<&Expr>) -> Vec<&Expr> {
    let Some(expr) = expr else { return Vec::new() };
    match bare(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            let mut out = conjuncts(Some(left));
            out.extend(conjuncts(Some(right)));
            out
        }
        other => vec![other],
    }
}
pub(super) fn projection(select: &Select) -> Option<Vec<&Expr>> {
    select
        .projection
        .iter()
        .map(|item| match item {
            SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => Some(expr),
            _ => None,
        })
        .collect()
}
pub(super) fn groups(select: &Select) -> &[Expr] {
    match &select.group_by {
        GroupByExpr::Expressions(exprs, modifiers) if modifiers.is_empty() => exprs,
        _ => &[],
    }
}
pub(super) fn select(query: &Query) -> Option<&Select> {
    match query.body.as_ref() {
        SetExpr::Select(select) => Some(select),
        _ => None,
    }
}
pub(super) fn args(function: &Function) -> Option<&FunctionArgumentList> {
    if function.filter.is_some() || function.over.is_some() || !function.within_group.is_empty() {
        return None;
    }
    match &function.args {
        FunctionArguments::List(args) if args.clauses.is_empty() => Some(args),
        _ => None,
    }
}
pub(super) fn function<'a>(expr: &'a Expr, name: &str) -> Option<&'a Function> {
    match bare(expr) {
        Expr::Function(function)
            if function.name.to_string().eq_ignore_ascii_case(name) && args(function).is_some() =>
        {
            Some(function)
        }
        _ => None,
    }
}
pub(super) fn argument(function: &Function) -> Option<&Expr> {
    let args = args(function)?;
    match args.args.as_slice() {
        [FunctionArg::Unnamed(FunctionArgExpr::Expr(expr))] => Some(expr),
        _ => None,
    }
}
pub(super) fn aggregate(expr: &Expr) -> bool {
    visit_expressions(expr, |expr| {
        if matches!(bare(expr), Expr::Function(f) if ["count","sum","avg","min","max","array_agg","json_agg","string_agg","bool_and","bool_or"].iter().any(|name| f.name.to_string().eq_ignore_ascii_case(name))) {
            ControlFlow::Break(())
        } else { ControlFlow::Continue(()) }
    }).is_break()
}
pub(super) fn pinned_value(expr: &Expr) -> bool {
    match bare(expr) {
        Expr::Value(value) => !matches!(value.value, Value::Null),
        Expr::Cast { expr, .. } => pinned_value(expr),
        _ => false,
    }
}
pub(super) fn uses(expr: &Expr, alias: &str) -> bool {
    visit_expressions(expr, |expr| {
        if column(expr).is_some_and(|(owner, _)| owner.as_deref() == Some(alias)) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    })
    .is_break()
}
pub(super) fn uses_output(expr: &Expr, alias: &str, names: &[String]) -> bool {
    visit_expressions(expr, |expr| {
        if column(expr).is_some_and(|(owner, name)| {
            owner.as_deref() == Some(alias) || owner.is_none() && names.contains(&name)
        }) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    })
    .is_break()
}
pub(super) fn output_names(select: &Select) -> Option<Vec<String>> {
    select
        .projection
        .iter()
        .map(|item| match item {
            SelectItem::UnnamedExpr(expr) => column(expr).map(|(_, name)| name),
            SelectItem::ExprWithAlias { alias, .. } => Some(ident(alias)),
            _ => None,
        })
        .collect()
}
