use super::super::select::uses::scope::{self, BaseRel};
use crate::codebase::postgres::idents::{
    ident_key, object_name_identity, relation_part_key, relation_suffix_key,
};
use sqlparser::ast::{Expr, TableFactor, TableWithJoins};

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

pub(super) fn owner(
    relations: &[BaseRel],
    qualifier: Option<&str>,
) -> (String, Option<Vec<String>>) {
    let Some(qualifier) = qualifier else {
        return (
            String::new(),
            (!relations.iter().any(|rel| rel.unknown))
                .then(|| relations.iter().map(|rel| rel.table.clone()).collect()),
        );
    };
    let mut matches = relations.iter().filter(|rel| {
        rel.alias.as_deref().map_or_else(
            || rel.table == qualifier || relation_suffix_key(&rel.table) == qualifier,
            |alias| alias == qualifier,
        )
    });
    match (matches.next(), matches.next()) {
        (Some(relation), None) if !relation.unknown => {
            (relation.table.clone(), Some(vec![relation.table.clone()]))
        }
        _ => (String::new(), None),
    }
}

pub(super) fn qualifier(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Nested(expr) => qualifier(expr),
        Expr::CompoundIdentifier(parts) => Some(
            parts[..parts.len() - 1]
                .iter()
                .map(|part| relation_part_key(&ident_key(part)))
                .collect::<Vec<_>>()
                .join("."),
        ),
        _ => None,
    }
}
