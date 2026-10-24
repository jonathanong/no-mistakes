use super::super::select::uses::scope::{self, BaseRel};
use crate::codebase::postgres::idents::{object_name_identity, relation_part_key};
use sqlparser::ast::{TableFactor, TableWithJoins};

pub(super) fn base_relations(tables: &[TableWithJoins], ctes: &[String]) -> Vec<BaseRel> {
    let mut out = Vec::new();
    for table in tables {
        push_factor(&table.relation, ctes, &mut out);
        for join in &table.joins {
            push_factor(&join.relation, ctes, &mut out);
        }
    }
    out
}
pub(super) fn push_factor(factor: &TableFactor, ctes: &[String], out: &mut Vec<BaseRel>) {
    if let TableFactor::NestedJoin {
        table_with_joins,
        alias: None,
    } = factor
    {
        out.extend(base_relations(std::slice::from_ref(table_with_joins), ctes));
        return;
    }
    let start = out.len();
    scope::push_factor(factor, ctes, out);
    if let TableFactor::Table { name, alias, .. } = factor {
        for relation in &mut out[start..] {
            relation.table = object_name_identity(name);
            relation.alias = alias.as_ref().map(|alias| {
                relation_part_key(&crate::codebase::postgres::idents::ident_key(&alias.name))
            });
        }
    }
    let unsafe_columns = match factor {
        TableFactor::Table { alias, args, .. } => {
            args.is_some()
                || alias
                    .as_ref()
                    .is_some_and(|alias| !alias.columns.is_empty())
        }
        TableFactor::NestedJoin { alias, .. } => alias.is_some(),
        _ => false,
    };
    if unsafe_columns {
        for relation in &mut out[start..] {
            relation.unknown = true;
        }
    }
}
