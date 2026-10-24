use super::super::super::predicates::base_table;
use crate::codebase::postgres::idents::{ident_key, object_name_key};
use sqlparser::ast::{TableFactor, TableWithJoins};

pub(in crate::codebase::postgres::statements) struct BaseRel {
    pub(in crate::codebase::postgres::statements) table: String,
    pub(in crate::codebase::postgres::statements) alias: Option<String>,
    pub(in crate::codebase::postgres::statements) unknown: bool,
}

pub(in crate::codebase::postgres::statements) fn base_relations(
    from: &[TableWithJoins],
    ctes: &[String],
) -> Vec<BaseRel> {
    let mut rels = Vec::new();
    for table in from {
        push_factor(&table.relation, ctes, &mut rels);
        for join in &table.joins {
            push_factor(&join.relation, ctes, &mut rels);
        }
    }
    rels
}

pub(in crate::codebase::postgres::statements) fn push_factor(
    factor: &TableFactor,
    ctes: &[String],
    rels: &mut Vec<BaseRel>,
) {
    match factor {
        TableFactor::Table { name, alias, .. } => {
            let Some(_) = base_table(name, ctes) else {
                rels.push(BaseRel {
                    table: String::new(),
                    alias: None,
                    unknown: true,
                });
                return;
            };
            rels.push(BaseRel {
                table: object_name_key(name),
                unknown: false,
                alias: alias.as_ref().map(|alias| ident_key(&alias.name)),
            });
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => rels.extend(base_relations(std::slice::from_ref(table_with_joins), ctes)),
        _ => rels.push(BaseRel {
            table: String::new(),
            alias: None,
            unknown: true,
        }),
    }
}
