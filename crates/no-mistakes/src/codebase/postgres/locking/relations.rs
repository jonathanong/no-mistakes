use super::single_row::pinned_columns;
use crate::codebase::postgres::catalog::normalize_table_name;
use sqlparser::ast::{
    LockClause, LockType, ObjectName, ObjectNamePart, SetExpr, TableFactor, TableWithJoins,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct LockedTables {
    pub(super) names: Vec<String>,
    pub(super) qualifiers: BTreeMap<String, Vec<String>>,
    /// Columns of each locked table pinned to one bound value by a top-level `WHERE` conjunct.
    pub(super) pinned: BTreeMap<String, Vec<String>>,
}

pub(super) fn locked_tables(body: &SetExpr, locks: &[LockClause]) -> Option<LockedTables> {
    let relations = relations(body)?;
    let update_locks: Vec<_> = locks
        .iter()
        .filter(|lock| lock.lock_type == LockType::Update)
        .collect();
    if update_locks.is_empty() {
        return Some(LockedTables {
            names: Vec::new(),
            qualifiers: BTreeMap::new(),
            pinned: BTreeMap::new(),
        });
    }
    let selection = select_selection(body);
    if update_locks.iter().any(|lock| lock.of.is_none()) {
        // Without `OF`, every FROM item is locked, so each must be a base table.
        return selected_tables(&relations, &relations.iter().collect::<Vec<_>>(), selection);
    }
    let mut tables = Vec::new();
    for lock in update_locks {
        let target = normalize(&qualified_relation_name(lock.of.as_ref()?));
        let matches = relations
            .iter()
            .filter(|relation| relation.names.contains(&target))
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return None;
        }
        tables.push(matches[0]);
    }
    selected_tables(&relations, &tables, selection)
}

fn select_selection(body: &SetExpr) -> Option<&sqlparser::ast::Expr> {
    match body {
        SetExpr::Select(select) => select.selection.as_ref(),
        _ => None,
    }
}

#[derive(Debug)]
pub(super) struct Relation {
    /// `None` for a derived table, lateral subquery, or table function: it can be
    /// joined but is not a base table a lock (or catalog key) can name.
    pub(super) table: Option<String>,
    pub(super) names: BTreeSet<String>,
}

fn relations(body: &SetExpr) -> Option<Vec<Relation>> {
    let SetExpr::Select(select) = body else {
        return None;
    };
    let mut relations = Vec::new();
    for table in &select.from {
        collect_table_with_joins(table, &mut relations)?;
    }
    (!relations.is_empty()).then_some(relations)
}

fn collect_table_with_joins(table: &TableWithJoins, relations: &mut Vec<Relation>) -> Option<()> {
    collect_table_factor(&table.relation, relations)?;
    for join in &table.joins {
        collect_table_factor(&join.relation, relations)?;
    }
    Some(())
}

fn collect_table_factor(factor: &TableFactor, relations: &mut Vec<Relation>) -> Option<()> {
    match factor {
        TableFactor::Table { name, alias, .. } => {
            let table = qualified_relation_name(name);
            let mut names = BTreeSet::from([
                normalize(&table),
                normalize(
                    &name
                        .0
                        .iter()
                        .rev()
                        .find_map(|part| match part {
                            ObjectNamePart::Identifier(ident) => Some(ident.to_string()),
                            _ => None,
                        })
                        .unwrap_or_default(),
                ),
            ]);
            if let Some(alias) = alias {
                names.insert(normalize(&alias.name.to_string()));
            }
            relations.push(Relation {
                table: Some(table),
                names,
            });
            Some(())
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => collect_table_with_joins(table_with_joins, relations),
        other => {
            relations.push(Relation {
                table: None,
                names: opaque_names(other),
            });
            Some(())
        }
    }
}

fn opaque_names(factor: &TableFactor) -> BTreeSet<String> {
    match factor {
        TableFactor::Derived { alias, .. }
        | TableFactor::Function { alias, .. }
        | TableFactor::TableFunction { alias, .. }
        | TableFactor::UNNEST { alias, .. } => alias
            .iter()
            .map(|alias| normalize(&alias.name.to_string()))
            .collect(),
        _ => BTreeSet::new(),
    }
}

fn selected_tables(
    all: &[Relation],
    selected: &[&Relation],
    selection: Option<&sqlparser::ast::Expr>,
) -> Option<LockedTables> {
    let mut qualifiers = BTreeMap::new();
    let mut pinned = BTreeMap::new();
    for relation in selected {
        let table = relation.table.clone()?;
        qualifiers
            .entry(table.clone())
            .or_insert_with(BTreeSet::new)
            .extend(relation.names.iter().cloned());
        pinned.insert(table, pinned_columns(all, relation, selection));
    }
    let names = qualifiers.keys().cloned().collect();
    let qualifiers = qualifiers
        .into_iter()
        .map(|(table, names)| (table, names.into_iter().collect()))
        .collect();
    Some(LockedTables {
        names,
        qualifiers,
        pinned,
    })
}

fn qualified_relation_name(name: &ObjectName) -> String {
    name.0
        .iter()
        .filter_map(|part| match part {
            ObjectNamePart::Identifier(ident) => Some(ident.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(".")
}

fn normalize(name: &str) -> String {
    normalize_table_name(name)
}
