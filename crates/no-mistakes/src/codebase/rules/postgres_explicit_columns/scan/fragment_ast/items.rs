use super::{FragmentStar, Rel, SYNTHETIC_TARGET};
use no_mistakes_postgres_sql::idents::{ident_key, object_name_key};
use sqlparser::ast::{
    Expr, FunctionArg, FunctionArgExpr, FunctionArguments, ObjectName, SelectItem,
    SelectItemQualifiedWildcardKind, Spanned, TableFactor,
};

pub(super) fn collect_items(
    items: &[SelectItem],
    rels: &[Rel],
    returning: bool,
    out: &mut Vec<FragmentStar>,
) {
    for item in items {
        let line = (item.span().start.line as usize).max(1);
        match item {
            SelectItem::Wildcard(_) => emit_bare(rels, returning, None, line, out),
            SelectItem::QualifiedWildcard(SelectItemQualifiedWildcardKind::ObjectName(name), _) => {
                emit_qualified(rels, name, returning, None, line, out);
            }
            SelectItem::UnnamedExpr(expr)
            | SelectItem::ExprWithAlias { expr, .. }
            | SelectItem::ExprWithAliases { expr, .. } => {
                walk_expr_stars(expr, rels, returning, line, out);
            }
            _ => {}
        }
    }
}

fn emit_bare(
    rels: &[Rel],
    returning: bool,
    func: Option<String>,
    line: usize,
    out: &mut Vec<FragmentStar>,
) {
    if rels.is_empty() {
        out.push(FragmentStar {
            returning,
            qualifier: None,
            relation: None,
            within_function: func,
            line,
        });
        return;
    }
    for rel in rels {
        if rel.is_local_derived {
            continue;
        }
        out.push(FragmentStar {
            returning,
            qualifier: None,
            relation: (rel.table != SYNTHETIC_TARGET).then(|| rel.table.clone()),
            within_function: func.clone(),
            line,
        });
    }
}

fn emit_qualified(
    rels: &[Rel],
    name: &ObjectName,
    returning: bool,
    func: Option<String>,
    line: usize,
    out: &mut Vec<FragmentStar>,
) {
    let ident = object_name_key(name);
    if ident.is_empty() {
        return;
    }
    let found = rels
        .iter()
        .find(|rel| rel.alias.as_deref() == Some(ident.as_str()))
        .or_else(|| {
            rels.iter().find(|rel| {
                rel.alias.is_none()
                    && (rel.table == ident || rel.table.rsplit('.').next() == Some(ident.as_str()))
            })
        });
    match found {
        Some(rel) if rel.is_local_derived => {}
        Some(rel) => out.push(FragmentStar {
            returning,
            qualifier: Some(ident),
            relation: (rel.table != SYNTHETIC_TARGET).then(|| rel.table.clone()),
            within_function: func,
            line,
        }),
        None => out.push(FragmentStar {
            returning,
            qualifier: Some(ident),
            relation: None,
            within_function: func,
            line,
        }),
    }
}

fn walk_expr_stars(
    expr: &Expr,
    rels: &[Rel],
    returning: bool,
    line: usize,
    out: &mut Vec<FragmentStar>,
) {
    let line = (expr.span().start.line as usize).max(line);
    match expr {
        Expr::Function(func) => {
            let name = object_name_key(&func.name);
            let FunctionArguments::List(args) = &func.args else {
                return;
            };
            for arg in &args.args {
                let inner = match arg {
                    FunctionArg::Unnamed(a)
                    | FunctionArg::Named { arg: a, .. }
                    | FunctionArg::ExprNamed { arg: a, .. } => a,
                };
                match inner {
                    FunctionArgExpr::Wildcard if !name.eq_ignore_ascii_case("count") => {
                        emit_bare(rels, returning, Some(name.clone()), line, out);
                    }
                    FunctionArgExpr::QualifiedWildcard(q) => {
                        emit_qualified(rels, q, returning, Some(name.clone()), line, out);
                    }
                    FunctionArgExpr::Expr(e) => walk_expr_stars(e, rels, returning, line, out),
                    _ => {}
                }
            }
        }
        Expr::Nested(inner) | Expr::Cast { expr: inner, .. } => {
            walk_expr_stars(inner, rels, returning, line, out);
        }
        Expr::Exists { subquery, .. } => super::collect_query(subquery, &[], true, out),
        Expr::Subquery(subquery) => super::collect_query(subquery, &[], false, out),
        _ => {}
    }
}

pub(super) fn collect_returning(
    target: &Rel,
    peers: &[Rel],
    items: Option<&[SelectItem]>,
    out: &mut Vec<FragmentStar>,
) {
    let Some(items) = items else { return };
    let all: Vec<_> = std::iter::once(target).chain(peers.iter()).cloned().collect();
    for item in items {
        let visible = if matches!(item, SelectItem::Wildcard(_)) {
            std::slice::from_ref(target)
        } else {
            &all
        };
        collect_items(std::slice::from_ref(item), visible, true, out);
    }
}

pub(super) fn target_rel(factor: &TableFactor) -> Option<Rel> {
    match factor {
        TableFactor::Table {
            name,
            alias,
            args: None,
            ..
        } => Some(Rel {
            table: object_name_key(name),
            alias: alias.as_ref().map(|a| ident_key(&a.name)),
            is_local_derived: false,
        }),
        _ => None,
    }
}

pub(super) fn walk_subqueries(expr: &Expr, ctes: &[String], out: &mut Vec<FragmentStar>) {
    match expr {
        Expr::Exists { subquery, .. } => super::collect_query(subquery, ctes, true, out),
        Expr::Subquery(subquery) => {
            super::collect_query(subquery, ctes, false, out);
        }
        Expr::BinaryOp { left, right, .. } => {
            walk_subqueries(left, ctes, out);
            walk_subqueries(right, ctes, out);
        }
        Expr::UnaryOp { expr: inner, .. } | Expr::Nested(inner) => {
            walk_subqueries(inner, ctes, out);
        }
        _ => {}
    }
}
