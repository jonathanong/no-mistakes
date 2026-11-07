use super::{LockWriteFacts, RowAccess};
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::Query;
use std::collections::BTreeSet;

pub(super) fn cte_names(query: &Query) -> BTreeSet<String> {
    query
        .with
        .iter()
        .flat_map(|with| &with.cte_tables)
        .map(|cte| ident_key(&cte.alias.name))
        .collect()
}

pub(super) fn hide_cte(row: &mut RowAccess, scopes: &[BTreeSet<String>]) {
    if row
        .relation
        .as_ref()
        .is_some_and(|name| scopes.iter().any(|names| names.contains(name)))
    {
        row.relation = None;
        row.key = None;
    }
}

pub(super) fn mark_self_contained(out: &mut LockWriteFacts, lock: usize, write: usize) {
    let relations: BTreeSet<_> = out.writes[write..]
        .iter()
        .filter_map(|write| write.relation.as_deref())
        .collect();
    for (index, lock) in out.locks.iter().enumerate().skip(lock) {
        if lock
            .relation
            .as_deref()
            .is_some_and(|relation| relations.contains(relation))
        {
            out.self_contained_locks.insert(index);
        }
    }
}
