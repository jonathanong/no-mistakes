use super::relation_name;
use sqlparser::ast::{LockClause, LockType, SetExpr, TableFactor, TableWithJoins};
use std::collections::BTreeSet;

pub(super) fn locked_tables(body: &SetExpr, locks: &[LockClause]) -> Option<Vec<String>> {
    let relations = relations(body)?;
    let update_locks: Vec<_> = locks
        .iter()
        .filter(|lock| lock.lock_type == LockType::Update)
        .collect();
    if update_locks.is_empty() {
        return Some(Vec::new());
    }
    if update_locks.iter().any(|lock| lock.of.is_none()) {
        return Some(distinct_tables(&relations));
    }
    let mut tables = BTreeSet::new();
    for lock in update_locks {
        let target = normalize(&relation_name(lock.of.as_ref()?));
        let matches = relations
            .iter()
            .filter(|relation| relation.names.contains(&target))
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return None;
        }
        tables.insert(matches[0].table.clone());
    }
    Some(tables.into_iter().collect())
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
            let table = relation_name(name);
            let mut names = BTreeSet::from([normalize(&table)]);
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

fn distinct_tables(relations: &[Relation]) -> Vec<String> {
    relations
        .iter()
        .map(|relation| relation.table.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn normalize(name: &str) -> String {
    name.to_ascii_lowercase()
}
