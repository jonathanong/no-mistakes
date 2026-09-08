use super::relation_name;
use sqlparser::ast::{
    LockClause, LockType, ObjectName, ObjectNamePart, SetExpr, TableFactor, TableWithJoins,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct LockedTables {
    pub(super) names: Vec<String>,
    pub(super) qualifiers: BTreeMap<String, Vec<String>>,
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
        });
    }
    if update_locks.iter().any(|lock| lock.of.is_none()) {
        return Some(selected_tables(&relations.iter().collect::<Vec<_>>()));
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
    Some(selected_tables(&tables))
}

#[derive(Debug)]
struct Relation {
    table: String,
    names: BTreeSet<String>,
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
            let mut names = BTreeSet::from([normalize(&table), normalize(&relation_name(name))]);
            if let Some(alias) = alias {
                names.insert(normalize(&alias.name.value));
            }
            relations.push(Relation { table, names });
            Some(())
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => collect_table_with_joins(table_with_joins, relations),
        _ => None,
    }
}

fn selected_tables(relations: &[&Relation]) -> LockedTables {
    let mut qualifiers = BTreeMap::new();
    for relation in relations {
        qualifiers
            .entry(relation.table.clone())
            .or_insert_with(BTreeSet::new)
            .extend(relation.names.iter().cloned());
    }
    let names = qualifiers.keys().cloned().collect();
    let qualifiers = qualifiers
        .into_iter()
        .map(|(table, names)| (table, names.into_iter().collect()))
        .collect();
    LockedTables { names, qualifiers }
}

fn qualified_relation_name(name: &ObjectName) -> String {
    name.0
        .iter()
        .filter_map(|part| match part {
            ObjectNamePart::Identifier(ident) => Some(ident.value.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(".")
}

fn normalize(name: &str) -> String {
    name.to_ascii_lowercase()
}
