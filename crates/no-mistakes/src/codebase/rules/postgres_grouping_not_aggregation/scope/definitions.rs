use super::super::expressions::ident;
use sqlparser::ast::Query;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) type Names = BTreeMap<usize, BTreeSet<String>>;

pub(crate) fn key(query: &Query) -> usize {
    std::ptr::from_ref(query) as usize
}

pub(crate) fn enter(
    query: &Query,
    inherited: Option<&BTreeSet<String>>,
    overrides: &mut Names,
) -> BTreeSet<String> {
    let inherited = overrides
        .remove(&key(query))
        .unwrap_or_else(|| inherited.cloned().unwrap_or_default());
    let mut body = inherited.clone();
    if let Some(with) = &query.with {
        body.extend(with.cte_tables.iter().map(|cte| ident(&cte.alias.name)));
        let mut visible = if with.recursive {
            body.clone()
        } else {
            inherited
        };
        for cte in &with.cte_tables {
            overrides.insert(key(&cte.query), visible.clone());
            visible.insert(ident(&cte.alias.name));
        }
    }
    body
}
