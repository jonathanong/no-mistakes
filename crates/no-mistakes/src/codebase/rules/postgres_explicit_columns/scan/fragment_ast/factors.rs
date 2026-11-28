use super::dml::{rel_table_key, visit_join_constraint, visit_table_args};
use super::{collect_query, FragmentStar, Rel, Scope};
use no_mistakes_postgres_sql::idents::ident_key;
use sqlparser::ast::{TableFactor, TableWithJoins};

pub(super) fn extend_outer(local: &[Rel], outer: &[Rel]) -> Vec<Rel> {
    let mut combined = Vec::with_capacity(local.len() + outer.len());
    combined.extend(local.iter().cloned().map(|mut r| {
        r.depth = 0;
        r
    }));
    combined.extend(outer.iter().cloned().map(|mut r| {
        r.depth += 1;
        r
    }));
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
            let outer = extend_outer(&rels, scope.outer_rels);
            let join_scope = Scope {
                ctes: scope.ctes,
                outer_rels: &outer,
            };
            visit_join_constraint(&join.join_operator, join_scope, out);
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
                if !rels.is_empty() {
                    let outer = extend_outer(rels, scope.outer_rels);
                    let lat_scope = Scope {
                        ctes: scope.ctes,
                        outer_rels: &outer,
                    };
                    visit_table_args(table_args, lat_scope, out);
                } else {
                    visit_table_args(table_args, scope, out);
                }
                if let Some(alias) = alias {
                    rels.push(Rel {
                        table: String::new(),
                        alias: Some(ident_key(&alias.name)),
                        join_aliases: Vec::new(),
                        is_local_derived: true,
                        hidden_by_join_alias: false,
                        depth: 0,
                    });
                }
                return;
            }
            let table = rel_table_key(name);
            let is_cte = name.0.len() == 1 && scope.ctes.iter().any(|cte| cte == &table);
            rels.push(Rel {
                table,
                alias: alias.as_ref().map(|a| ident_key(&a.name)),
                join_aliases: Vec::new(),
                is_local_derived: is_cte,
                hidden_by_join_alias: false,
                depth: 0,
            });
        }
        TableFactor::Derived {
            lateral,
            subquery,
            alias,
            ..
        } => {
            if *lateral && !rels.is_empty() {
                let outer = extend_outer(rels, scope.outer_rels);
                let lat_scope = Scope {
                    ctes: scope.ctes,
                    outer_rels: &outer,
                };
                collect_query(subquery, lat_scope, false, out);
            } else {
                collect_query(subquery, scope, false, out);
            }
            rels.push(Rel {
                table: String::new(),
                alias: alias.as_ref().map(|a| ident_key(&a.name)),
                join_aliases: Vec::new(),
                is_local_derived: true,
                hidden_by_join_alias: false,
                depth: 0,
            });
        }
        TableFactor::NestedJoin {
            table_with_joins,
            alias,
        } => {
            let mut inner = collect_rels(std::slice::from_ref(table_with_joins), scope, out);
            if let Some(alias) = alias {
                let name = ident_key(&alias.name);
                for r in &mut inner {
                    r.join_aliases.push(name.clone());
                    r.hidden_by_join_alias = true;
                }
            }
            rels.extend(inner);
        }
        _ => {}
    }
}
