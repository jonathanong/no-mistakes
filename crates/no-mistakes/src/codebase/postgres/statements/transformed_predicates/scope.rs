use super::super::select::uses::scope::{self, BaseRel};
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
