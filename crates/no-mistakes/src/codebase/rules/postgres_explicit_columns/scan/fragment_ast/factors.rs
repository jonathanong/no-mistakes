use super::dml::{visit_join_constraint, visit_table_args};
use super::{collect_query, FragmentStar, Rel, Scope};
use no_mistakes_postgres_sql::idents::{ident_key, object_name_key};
use sqlparser::ast::{TableFactor, TableWithJoins};

pub(super) fn extend_outer(local: &[Rel], outer: &[Rel]) -> Vec<Rel> {
    let mut combined = Vec::with_capacity(local.len() + outer.len());
    combined.extend_from_slice(local);
    combined.extend_from_slice(outer);
    combined
}

pub(super) fn collect_rels(
    from: &[TableWithJoins],
    scope: Scope<'_>,
    out: &mut Vec<FragmentStar>,
) -> Vec<Rel> {
    let mut rels = Vec::new();
    for table in from {
        push_factor(&table.relation, scope, &mut rels, out);
        for join in &table.joins {
            push_factor(&join.relation, scope, &mut rels, out);
            visit_join_constraint(&join.join_operator, scope, out);
        }
    }
    rels
}

pub(super) fn push_factor(
    factor: &TableFactor,
    scope: Scope<'_>,
    rels: &mut Vec<Rel>,
    out: &mut Vec<FragmentStar>,
) {
    match factor {
        TableFactor::Table {
            name, alias, args, ..
        } => {
            if let Some(table_args) = args {
                visit_table_args(table_args, scope, out);
                if let Some(alias) = alias {
                    rels.push(Rel {
                        table: String::new(),
                        alias: Some(ident_key(&alias.name)),
                        join_aliases: Vec::new(),
                        is_local_derived: true,
                    });
                }
                return;
            }
            let table = object_name_key(name);
            let is_cte = !table.contains('.') && scope.ctes.iter().any(|cte| cte == &table);
            rels.push(Rel {
                table,
                alias: alias.as_ref().map(|a| ident_key(&a.name)),
                join_aliases: Vec::new(),
                is_local_derived: is_cte,
            });
        }
        TableFactor::Derived {
            subquery, alias, ..
        } => {
            collect_query(subquery, scope, false, out);
            rels.push(Rel {
                table: String::new(),
                alias: alias.as_ref().map(|a| ident_key(&a.name)),
                join_aliases: Vec::new(),
                is_local_derived: true,
            });
        }
        TableFactor::NestedJoin {
            table_with_joins,
            alias,
        } => {
            let mut inner = collect_rels(std::slice::from_ref(table_with_joins), scope, out);
            if let Some(alias) = alias {
                let name = ident_key(&alias.name);
                inner
                    .iter_mut()
                    .for_each(|r| r.join_aliases.push(name.clone()));
            }
            rels.extend(inner);
        }
        _ => {}
    }
}
