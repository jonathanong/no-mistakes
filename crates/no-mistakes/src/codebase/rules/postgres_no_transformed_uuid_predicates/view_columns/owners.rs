use crate::codebase::postgres::idents::{ident_key, object_name_identity, relation_part_key};
use sqlparser::ast::{Join, TableFactor};
use std::collections::BTreeSet;

pub(super) fn joined(joins: &[Join]) -> Option<BTreeSet<String>> {
    let mut owners = BTreeSet::new();
    let mut factors: Vec<_> = joins.iter().map(|join| &join.relation).collect();
    while let Some(factor) = factors.pop() {
        let alias = match factor {
            TableFactor::Table { name, alias, .. } => {
                if alias.is_none() {
                    owners.insert(object_name_identity(name));
                    owners.insert(relation_part_key(&ident_key(name.0.last()?.as_ident()?)));
                }
                alias
            }
            TableFactor::Derived { alias, .. } => alias,
            TableFactor::NestedJoin {
                table_with_joins,
                alias,
            } => {
                if alias.is_none() {
                    factors.push(&table_with_joins.relation);
                    factors.extend(table_with_joins.joins.iter().map(|join| &join.relation));
                }
                alias
            }
            _ => return None,
        };
        if let Some(alias) = alias {
            owners.insert(relation_part_key(&ident_key(&alias.name)));
        }
    }
    Some(owners)
}
