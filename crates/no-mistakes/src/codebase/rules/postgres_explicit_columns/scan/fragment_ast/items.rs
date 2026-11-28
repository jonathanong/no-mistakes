use super::exprs::{walk_expr_stars, walk_subqueries};
use super::{FragmentStar, Rel, SYNTHETIC_TARGET};
use no_mistakes_postgres_sql::idents::{ident_key, object_name_key};
use sqlparser::ast::{
    JoinConstraint, JoinOperator, ObjectName, SelectItem, SelectItemQualifiedWildcardKind, Spanned,
    TableFactor,
};

pub(super) fn collect_items(
    items: &[SelectItem],
    rels: &[Rel],
    ctes: &[String],
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
                walk_expr_stars(expr, rels, ctes, returning, line, out);
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

pub(super) fn emit_qualified(
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

pub(super) fn collect_returning(
    target: &Rel,
    peers: &[Rel],
    ctes: &[String],
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
        collect_items(std::slice::from_ref(item), visible, ctes, true, out);
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

pub(super) fn visit_join_constraint(
    op: &JoinOperator,
    ctes: &[String],
    out: &mut Vec<FragmentStar>,
) {
    let constraint = match op {
        JoinOperator::Join(c)
        | JoinOperator::Inner(c)
        | JoinOperator::Left(c)
        | JoinOperator::LeftOuter(c)
        | JoinOperator::Right(c)
        | JoinOperator::RightOuter(c)
        | JoinOperator::FullOuter(c)
        | JoinOperator::LeftSemi(c)
        | JoinOperator::RightSemi(c)
        | JoinOperator::LeftAnti(c)
        | JoinOperator::RightAnti(c)
        | JoinOperator::StraightJoin(c) => Some(c),
        _ => None,
    };
    if let Some(JoinConstraint::On(expr)) = constraint {
        walk_subqueries(expr, ctes, out);
    }
}
