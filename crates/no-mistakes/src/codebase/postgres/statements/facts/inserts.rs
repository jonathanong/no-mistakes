use super::super::value::PlaceholderPositions;
use super::{insert, SqlInsertFact};
use sqlparser::ast::{Query, SetExpr, Statement};
#[cfg(test)]
mod tests;

pub(super) fn collect_query_inserts(
    sources: &super::super::lines::InsertSources<'_>,
    query: &Query,
    insert_n: &mut usize,
    inserts: &mut Vec<SqlInsertFact>,
    placeholder_positions: PlaceholderPositions<'_>,
    policy_arbiters: &crate::fx::FxHashMap<
        sqlparser::tokenizer::Location,
        super::SqlConflictArbiter,
    >,
) {
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            collect_query_inserts(
                sources,
                &cte.query,
                insert_n,
                inserts,
                placeholder_positions,
                policy_arbiters,
            );
        }
    }
    collect_set_inserts(
        sources,
        &query.body,
        insert_n,
        inserts,
        placeholder_positions,
        policy_arbiters,
    );
}

fn collect_set_inserts(
    sources: &super::super::lines::InsertSources<'_>,
    expr: &SetExpr,
    insert_n: &mut usize,
    inserts: &mut Vec<SqlInsertFact>,
    placeholder_positions: PlaceholderPositions<'_>,
    policy_arbiters: &crate::fx::FxHashMap<
        sqlparser::tokenizer::Location,
        super::SqlConflictArbiter,
    >,
) {
    match expr {
        SetExpr::Insert(statement) => {
            if matches!(statement, Statement::Insert(_)) {
                *insert_n += 1;
                if let Some(mut fact) =
                    insert::from_statement_at(sources, statement, *insert_n, placeholder_positions)
                {
                    if let Statement::Insert(insert) = statement {
                        insert::apply_policy_arbiter(insert, &mut fact, policy_arbiters);
                    }
                    inserts.push(fact);
                }
            }
        }
        SetExpr::Query(query) => collect_query_inserts(
            sources,
            query,
            insert_n,
            inserts,
            placeholder_positions,
            policy_arbiters,
        ),
        SetExpr::SetOperation { left, right, .. } => {
            collect_set_inserts(
                sources,
                left,
                insert_n,
                inserts,
                placeholder_positions,
                policy_arbiters,
            );
            collect_set_inserts(
                sources,
                right,
                insert_n,
                inserts,
                placeholder_positions,
                policy_arbiters,
            );
        }
        _ => {}
    }
}
