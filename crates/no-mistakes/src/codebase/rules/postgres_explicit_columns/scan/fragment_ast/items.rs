use super::exprs::walk_expr_stars;
use super::{FragmentStar, Rel, Scope, SYNTHETIC_TARGET};
use no_mistakes_postgres_sql::idents::object_name_key;
use sqlparser::ast::{ObjectName, SelectItem, SelectItemQualifiedWildcardKind, Spanned};

pub(super) fn collect_items(
    items: &[SelectItem],
    rels: &[Rel],
    scope: Scope<'_>,
    returning: bool,
    out: &mut Vec<FragmentStar>,
) {
    for item in items {
        let line = (item.span().start.line as usize).max(1);
        match item {
            SelectItem::Wildcard(_) => emit_bare(rels, returning, None, line, out),
            SelectItem::QualifiedWildcard(SelectItemQualifiedWildcardKind::ObjectName(name), _) => {
                emit_qualified(rels, scope.outer_rels, name, returning, None, line, out);
            }
            SelectItem::UnnamedExpr(expr)
            | SelectItem::ExprWithAlias { expr, .. }
            | SelectItem::ExprWithAliases { expr, .. } => {
                walk_expr_stars(expr, rels, scope, returning, None, line, out);
            }
            _ => {}
        }
    }
}

pub(super) fn emit_bare(
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
        if !rel.is_local_derived {
            out.push(FragmentStar {
                returning,
                qualifier: None,
                relation: (rel.table != SYNTHETIC_TARGET).then(|| rel.table.clone()),
                within_function: func.clone(),
                line,
            });
        }
    }
}

pub(super) fn emit_qualified(
    rels: &[Rel],
    outer_rels: &[Rel],
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
    if resolve_in(rels, &ident, returning, &func, line, out)
        || resolve_in(outer_rels, &ident, returning, &func, line, out)
    {
        return;
    }
    out.push(FragmentStar {
        returning,
        qualifier: Some(ident),
        relation: None,
        within_function: func,
        line,
    });
}

fn resolve_in(
    rels: &[Rel],
    ident: &str,
    returning: bool,
    func: &Option<String>,
    line: usize,
    out: &mut Vec<FragmentStar>,
) -> bool {
    let found = rels
        .iter()
        .find(|rel| rel.alias.as_deref() == Some(ident))
        .or_else(|| {
            rels.iter().find(|rel| {
                rel.alias.is_none()
                    && (rel.table == ident || rel.table.rsplit('.').next() == Some(ident))
            })
        });
    if let Some(rel) = found {
        if !rel.is_local_derived {
            out.push(FragmentStar {
                returning,
                qualifier: Some(ident.to_string()),
                relation: (rel.table != SYNTHETIC_TARGET).then(|| rel.table.clone()),
                within_function: func.clone(),
                line,
            });
        }
        return true;
    }
    let joined: Vec<_> = rels
        .iter()
        .filter(|rel| rel.join_aliases.iter().any(|a| a == ident))
        .collect();
    if !joined.is_empty() {
        for rel in joined {
            if !rel.is_local_derived {
                out.push(FragmentStar {
                    returning,
                    qualifier: Some(ident.to_string()),
                    relation: (rel.table != SYNTHETIC_TARGET).then(|| rel.table.clone()),
                    within_function: func.clone(),
                    line,
                });
            }
        }
        return true;
    }
    false
}

pub(super) fn collect_returning(
    target: &Rel,
    peers: &[Rel],
    scope: Scope<'_>,
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
        collect_items(std::slice::from_ref(item), visible, scope, true, out);
    }
}
