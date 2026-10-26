use super::inserts::collect_query_inserts;
use super::*;

pub(super) struct FactOut<'a> {
    pub(super) policy_arbiters:
        &'a crate::fx::FxHashMap<sqlparser::tokenizer::Location, SqlConflictArbiter>,
    pub(super) insert_sources: &'a super::lines::InsertSources<'a>,
    pub(super) insert_n: &'a mut usize,
    pub(super) inserts: &'a mut Vec<SqlInsertFact>,
    pub(super) selects: &'a mut Vec<SqlSelectFact>,
    pub(super) updates: &'a mut Vec<Vec<SqlRelationPredicateFact>>,
    pub(super) deletes: &'a mut Vec<Vec<SqlRelationPredicateFact>>,
    pub(super) triggers: &'a mut Vec<SqlTriggerFact>,
    pub(super) returning_stars: &'a mut Vec<SqlStarProjectionFact>,
    pub(super) mutation_column_uses: &'a mut Vec<SqlColumnUseFact>,
}

pub(super) fn collect_one(
    sql: &str,
    tokens: &[sqlparser::tokenizer::TokenWithSpan],
    statement: &Statement,
    placeholder_positions: super::value::PlaceholderPositions<'_>,
    table_cursor: Option<&mut bounds::TableTokenCursor>,
    out: &mut FactOut<'_>,
) {
    if let Statement::Insert(insert) = statement {
        *out.insert_n += 1;
        if let Some(mut fact) = insert::from_statement_at(
            out.insert_sources,
            statement,
            *out.insert_n,
            placeholder_positions,
        ) {
            insert::apply_policy_arbiter(insert, &mut fact, out.policy_arbiters);
            out.inserts.push(fact);
        }
        if let Some(source) = insert.source.as_deref() {
            collect_query_inserts(
                out.insert_sources,
                source,
                out.insert_n,
                out.inserts,
                placeholder_positions,
                out.policy_arbiters,
            );
        }
    }
    if matches!(statement, Statement::CreateTrigger(_)) {
        if let Some(fact) = trigger::from_statement(tokens, statement) {
            out.triggers.push(fact);
        }
    }
    if matches!(statement, Statement::DropTrigger(_)) {
        if let Some(fact) = trigger::from_drop(tokens, statement) {
            out.triggers.push(fact);
        }
    }
    if let Statement::Query(query) = statement {
        collect_query_inserts(
            out.insert_sources,
            query,
            out.insert_n,
            out.inserts,
            placeholder_positions,
            out.policy_arbiters,
        );
    }
    select::collect_with_placeholder_positions(sql, statement, placeholder_positions, out.selects);
    select::collect_table_arms(statement, table_cursor, out.selects);
    out.returning_stars
        .extend(select::returning_stars(sql, statement));
    mutations::collect(
        sql,
        statement,
        out.updates,
        out.deletes,
        out.selects,
        out.mutation_column_uses,
        placeholder_positions,
    );
}
