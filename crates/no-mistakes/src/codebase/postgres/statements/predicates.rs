mod constrain;

#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod tests;

use super::SqlRelationPredicateFact;
use crate::codebase::postgres::idents::ident_key;
use constrain::Instance;
use sqlparser::ast::{
    Expr, JoinOperator, ObjectName, ObjectNamePart, Select, Spanned, TableFactor, TableWithJoins,
};

pub(super) fn select_relations(
    sql: &str,
    select: &Select,
    ctes: &[String],
) -> Vec<SqlRelationPredicateFact> {
    relations_for(sql, &select.from, select.selection.as_ref(), ctes)
}

pub(super) fn write_relations(
    sql: &str,
    tables: &[TableWithJoins],
    selection: Option<&Expr>,
    ctes: &[String],
) -> Vec<SqlRelationPredicateFact> {
    relations_for(sql, tables, selection, ctes)
}

/// Base table name when `name` is not an in-scope CTE. Schema-qualified
/// references stay tables even if a CTE reuses the unqualified name.
pub(super) fn base_table(name: &ObjectName, ctes: &[String]) -> Option<String> {
    let idents = name
        .0
        .iter()
        .filter(|part| matches!(part, ObjectNamePart::Identifier(_)))
        .count();
    let table = crate::codebase::postgres::idents::object_name_key(name);
    if table.is_empty() {
        return None;
    }
    if idents <= 1 && is_cte(&table, ctes) {
        None
    } else {
        Some(table)
    }
}

fn relations_for(
    _sql: &str,
    tables: &[TableWithJoins],
    selection: Option<&Expr>,
    ctes: &[String],
) -> Vec<SqlRelationPredicateFact> {
    let mut gathered = Gather::default();
    for table in tables {
        let start = gathered.instances.len();
        walk_relation(&table.relation, false, None, ctes, &mut gathered);
        for join in &table.joins {
            let right = gathered.instances.len();
            let on = super::select::join_expr(&join.join_operator);
            // ON filters only the non-preserved side of an outer join.
            if matches!(
                join.join_operator,
                JoinOperator::Join(_)
                    | JoinOperator::Inner(_)
                    | JoinOperator::Right(_)
                    | JoinOperator::RightOuter(_)
            ) {
                if let Some(on) = on {
                    for instance in &mut gathered.instances[start..right] {
                        instance.extra_on.push(on);
                    }
                }
            }
            let constrained_right = matches!(
                join.join_operator,
                JoinOperator::Join(_)
                    | JoinOperator::Inner(_)
                    | JoinOperator::Left(_)
                    | JoinOperator::LeftOuter(_)
            );
            walk_relation(&join.relation, constrained_right, on, ctes, &mut gathered);
        }
    }
    let constrained = constrain::constrain(
        &gathered.instances,
        selection,
        gathered.from_items,
        gathered.all_base,
    );
    gathered
        .instances
        .iter()
        .zip(constrained)
        .map(
            |(instance, (constrained_columns, unqualified_columns))| SqlRelationPredicateFact {
                line: instance.line,
                table: instance.table.clone(),
                alias: instance.alias.clone(),
                constrained_columns,
                unqualified_columns,
            },
        )
        .collect()
}

struct Gather<'a> {
    instances: Vec<Instance<'a>>,
    from_items: usize,
    all_base: bool,
}

impl Default for Gather<'_> {
    fn default() -> Self {
        Self {
            instances: Vec::new(),
            from_items: 0,
            all_base: true,
        }
    }
}

fn walk_relation<'a>(
    factor: &'a TableFactor,
    joined: bool,
    on: Option<&'a Expr>,
    ctes: &[String],
    gathered: &mut Gather<'a>,
) {
    match factor {
        TableFactor::Table {
            name, alias, args, ..
        } => {
            gathered.from_items += 1;
            if args.is_some() {
                gathered.all_base = false;
                return;
            }
            let Some(table) = base_table(name, ctes) else {
                gathered.all_base = false;
                return;
            };
            gathered.instances.push(Instance {
                table,
                alias: alias.as_ref().map(|alias| ident_key(&alias.name)),
                line: name.span().start.line as usize,
                extra_on: Vec::new(),
                joined,
                on,
            });
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => {
            walk_relation(&table_with_joins.relation, joined, on, ctes, gathered);
            for join in &table_with_joins.joins {
                walk_relation(
                    &join.relation,
                    true,
                    super::select::join_expr(&join.join_operator),
                    ctes,
                    gathered,
                );
            }
        }
        _ => {
            gathered.from_items += 1;
            gathered.all_base = false;
        }
    }
}

fn is_cte(table: &str, ctes: &[String]) -> bool {
    ctes.iter().any(|cte| cte == table)
}
