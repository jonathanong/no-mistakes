mod constrain;
mod projection;
pub(crate) use projection::{Identity, Projection};

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

pub(super) fn select_relations_projected(
    sql: &str,
    select: &Select,
    ctes: &[String],
    positions: super::value::PlaceholderPositions<'_>,
    projection: &impl Projection,
) -> Vec<SqlRelationPredicateFact> {
    relations_for(
        sql,
        &select.from,
        select.selection.as_ref(),
        ctes,
        positions,
        projection,
    )
}

pub(super) fn write_relations(
    sql: &str,
    tables: &[TableWithJoins],
    selection: Option<&Expr>,
    ctes: &[String],
    positions: super::value::PlaceholderPositions<'_>,
) -> Vec<SqlRelationPredicateFact> {
    relations_for(sql, tables, selection, ctes, positions, &Identity)
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
    positions: super::value::PlaceholderPositions<'_>,
    projection: &impl Projection,
) -> Vec<SqlRelationPredicateFact> {
    let mut gathered = Gather::default();
    for table in tables {
        let start = gathered.instances.len();
        walk_relation(
            &table.relation,
            false,
            None,
            ctes,
            &mut gathered,
            projection,
        );
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
            walk_relation(
                &join.relation,
                constrained_right,
                on,
                ctes,
                &mut gathered,
                projection,
            );
        }
    }
    let constrained = constrain::constrain_projected(
        &gathered.instances,
        selection,
        gathered.from_items,
        gathered.all_base,
        positions,
        projection,
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

mod gather;
use gather::{walk_relation, Gather};

fn is_cte(table: &str, ctes: &[String]) -> bool {
    ctes.iter().any(|cte| cte == table)
}
