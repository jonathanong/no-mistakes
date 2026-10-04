//! Relation references throughout a view query, including scalar subqueries.
use crate::codebase::postgres::decoded_parts;
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{ObjectName, Query, TableFactor, Visit, Visitor};
use std::collections::{BTreeSet, HashSet};
use std::ops::ControlFlow;

#[derive(Default)]
struct Relations {
    ctes: Vec<CteScope>,
    names: BTreeSet<String>,
    table_functions: HashSet<*const ObjectName>,
}

struct CteScope {
    aliases: Vec<String>,
    queries: Vec<*const Query>,
    visible: usize,
    recursive: bool,
    restore_parent_visible: Option<usize>,
}

impl Visitor for Relations {
    type Break = ();

    fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<Self::Break> {
        if let TableFactor::Table {
            name,
            args: Some(_),
            ..
        } = factor
        {
            self.table_functions.insert(name as *const ObjectName);
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<Self::Break> {
        let mut restore_parent_visible = None;
        if let Some(parent) = self.ctes.last_mut() {
            if let Some(index) = parent
                .queries
                .iter()
                .position(|candidate| std::ptr::eq(*candidate, query as *const Query))
            {
                restore_parent_visible = Some(parent.visible);
                // Recursive WITH allows forward aliases; nonrecursive WITH sees
                // only earlier aliases, so a later name may still mean a table.
                parent.visible = if parent.recursive {
                    parent.aliases.len()
                } else {
                    index
                };
            }
        }
        let ctes = query.with.as_ref().map(|with| &with.cte_tables);
        let aliases = ctes
            .into_iter()
            .flatten()
            .map(|cte| ident_key(&cte.alias.name))
            .collect::<Vec<_>>();
        let queries = ctes
            .into_iter()
            .flatten()
            .map(|cte| cte.query.as_ref() as *const Query)
            .collect();
        self.ctes.push(CteScope {
            visible: aliases.len(),
            aliases,
            queries,
            recursive: query.with.as_ref().is_some_and(|with| with.recursive),
            restore_parent_visible,
        });
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<Self::Break> {
        if let Some(visible) = self
            .ctes
            .pop()
            .and_then(|scope| scope.restore_parent_visible)
        {
            if let Some(parent) = self.ctes.last_mut() {
                parent.visible = visible;
            }
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_relation(&mut self, relation: &ObjectName) -> ControlFlow<Self::Break> {
        if self
            .table_functions
            .contains(&(relation as *const ObjectName))
        {
            return ControlFlow::Continue(());
        }
        let name = super::sql_name(relation);
        let parts = decoded_parts(&name);
        let is_cte = parts.len() == 1
            && self
                .ctes
                .iter()
                .rev()
                .any(|scope| scope.aliases[..scope.visible].contains(&parts[0]));
        if !is_cte {
            self.names.insert(name);
        }
        ControlFlow::Continue(())
    }
}

pub(in crate::codebase::postgres::statements::bounds) fn names(query: &Query) -> BTreeSet<String> {
    let mut relations = Relations::default();
    let _ = query.visit(&mut relations);
    relations.names
}

#[cfg(test)]
mod tests;
