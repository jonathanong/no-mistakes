use super::inserts::collect_query_inserts;
use super::*;

pub(super) struct FactOut<'a> {
    pub(super) insert_n: &'a mut usize,
    pub(super) trigger_n: &'a mut usize,
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
    statement: &Statement,
    placeholder_positions: super::value::PlaceholderPositions<'_>,
    out: &mut FactOut<'_>,
) {
    if let Statement::Insert(insert) = statement {
        *out.insert_n += 1;
        if let Some(fact) =
            insert::from_statement_at(sql, statement, *out.insert_n, placeholder_positions)
        {
            out.inserts.push(fact);
        }
        if let Some(source) = insert.source.as_deref() {
            collect_query_inserts(
                sql,
                source,
                out.insert_n,
                out.inserts,
                placeholder_positions,
            );
        }
    }
    if matches!(statement, Statement::CreateTrigger(_)) {
        *out.trigger_n += 1;
        if let Some(fact) = trigger::from_statement(sql, statement, *out.trigger_n) {
            out.triggers.push(fact);
        }
    }
    if let Statement::Query(query) = statement {
        collect_query_inserts(sql, query, out.insert_n, out.inserts, placeholder_positions);
    }
    select::collect_with_placeholder_positions(sql, statement, placeholder_positions, out.selects);
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
