use crate::codebase::postgres::idents::{ident_key, object_name_key, unwrap_expr};
use crate::codebase::postgres::statements::SqlBoundQuery;
use sqlparser::ast::{
    Cte, CteAsMaterialized, Expr, Select, SelectItem, SetExpr, SetOperator, SetQuantifier,
    TableFactor,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn stable_donors(ctes: &[Cte]) -> BTreeSet<String> {
    let mut names = BTreeMap::new();
    for cte in ctes {
        let stable = cte.materialized != Some(CteAsMaterialized::NotMaterialized);
        names
            .entry(ident_key(&cte.alias.name))
            .and_modify(|stable| *stable = false)
            .or_insert(stable);
    }
    names
        .into_iter()
        .filter_map(|(name, stable)| stable.then_some(name))
        .collect()
}

pub(super) fn project(cte: &Cte, donors: &BTreeSet<String>, bound: &mut SqlBoundQuery) {
    if cte.query.with.is_some() || bound.items.len() != 2 {
        return;
    }
    let SetExpr::SetOperation {
        op: SetOperator::Union,
        set_quantifier: SetQuantifier::All,
        left,
        right,
    } = &*cte.query.body
    else {
        return;
    };
    let (SetExpr::Select(anchor), SetExpr::Select(step)) = (&**left, &**right) else {
        return;
    };
    let name = ident_key(&cte.alias.name);
    let (Some(anchor_sources), Some(step_sources)) =
        (sources(anchor, &name, donors), sources(step, &name, donors))
    else {
        return;
    };
    if anchor_sources.recursive != 0
        || step_sources.recursive != 1
        || anchor_sources.ctes.len() != 1
        || anchor_sources.ctes != step_sources.ctes
    {
        return;
    }
    for ((output, anchor), step) in bound
        .outputs
        .iter_mut()
        .zip(&anchor.projection)
        .zip(&step.projection)
    {
        let (Some(anchor_origin), Some(step_origin)) =
            (origin(anchor, &anchor_sources), origin(step, &step_sources))
        else {
            continue;
        };
        if anchor_origin == step_origin {
            output.recursive_domain = true;
        }
    }
}

struct Sources {
    aliases: BTreeMap<String, Option<String>>,
    ctes: BTreeSet<String>,
    recursive: usize,
}

fn sources(select: &Select, recursive: &str, donors: &BTreeSet<String>) -> Option<Sources> {
    let mut found = Sources {
        aliases: BTreeMap::new(),
        ctes: BTreeSet::new(),
        recursive: 0,
    };
    for factor in select.from.iter().flat_map(|from| {
        std::iter::once(&from.relation).chain(from.joins.iter().map(|join| &join.relation))
    }) {
        let (alias, origin) = match factor {
            TableFactor::Table {
                name,
                alias,
                args: None,
                ..
            } if name.0.len() == 1 && alias.as_ref().is_none_or(|a| a.columns.is_empty()) => {
                let name = object_name_key(name);
                let alias = alias
                    .as_ref()
                    .map_or_else(|| name.clone(), |a| ident_key(&a.name));
                if name == recursive {
                    found.recursive += 1;
                    (alias, None)
                } else if donors.contains(&name) {
                    found.ctes.insert(name.clone());
                    (alias, Some(name))
                } else {
                    return None;
                }
            }
            TableFactor::Derived {
                lateral: true,
                alias: Some(alias),
                subquery,
                ..
            } if !super::super::temporary::view_relations::names(subquery).contains(recursive) => {
                (ident_key(&alias.name), None)
            }
            _ => return None,
        };
        if found.aliases.insert(alias, origin).is_some() {
            return None;
        }
    }
    Some(found)
}

fn origin(item: &SelectItem, sources: &Sources) -> Option<(String, String)> {
    let expression = match item {
        SelectItem::ExprWithAlias { expr, .. } | SelectItem::UnnamedExpr(expr) => expr,
        _ => return None,
    };
    let (source, column) = match unwrap_expr(expression) {
        Expr::CompoundIdentifier(parts) if parts.len() == 2 => (
            sources.aliases.get(&ident_key(&parts[0]))?.as_ref()?,
            ident_key(&parts[1]),
        ),
        Expr::Identifier(ident) if sources.aliases.len() == 1 => {
            (sources.aliases.values().next()?.as_ref()?, ident_key(ident))
        }
        _ => return None,
    };
    Some((source.clone(), column))
}
