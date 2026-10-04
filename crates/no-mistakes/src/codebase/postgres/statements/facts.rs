use super::tokens::Tokens;
use super::*;
mod inserts;
use crate::codebase::postgres::parse::{parse_postgres_sql, parse_postgres_sql_lenient};
use inserts::collect_query_inserts;
use sqlparser::ast::{Spanned, Statement};

/// Extract INSERT/SELECT/trigger facts from one SQL source.
pub fn extract_sql_statement_facts(sql: &str) -> SqlStatementFileFacts {
    extract_sql_statement_facts_with_bounds(sql, true)
}

/// Extract the requested SQL projections without reparsing for row bounds.
pub(crate) fn extract_sql_statement_facts_with_bounds(
    sql: &str,
    collect_bounds: bool,
) -> SqlStatementFileFacts {
    extract_sql_statement_facts_with_placeholder_positions(sql, collect_bounds, None)
}

/// Extract facts while identifying the SQL-local positions of recovered interpolation markers.
pub(crate) fn extract_sql_statement_facts_with_recovered_placeholders(
    sql: &str,
    collect_bounds: bool,
    recovered_placeholder_positions: &[(u32, u32)],
) -> SqlStatementFileFacts {
    extract_sql_statement_facts_with_placeholder_positions(
        sql,
        collect_bounds,
        Some(recovered_placeholder_positions),
    )
}

fn extract_sql_statement_facts_with_placeholder_positions(
    sql: &str,
    collect_bounds: bool,
    placeholder_positions: super::value::PlaceholderPositions<'_>,
) -> SqlStatementFileFacts {
    let parsed = parse_postgres_sql(sql);
    let parse_failed = parsed.is_err();
    let statements = parsed.unwrap_or_else(|_| parse_postgres_sql_lenient(sql));
    extract_from_parsed_with_recovered_placeholders(
        sql,
        &statements,
        parse_failed,
        collect_bounds,
        placeholder_positions,
    )
}

pub(crate) fn extract_from_parsed_with_recovered_placeholders(
    sql: &str,
    statements: &[Statement],
    parse_failed: bool,
    collect_bounds: bool,
    placeholder_positions: super::value::PlaceholderPositions<'_>,
) -> SqlStatementFileFacts {
    let masked = fallback::mask_quoted_sql(sql);
    let insert_keyword_count = fallback::insert_keyword_count(&masked);
    let mut writes = Vec::new();
    let mut inserts = Vec::new();
    let mut selects = Vec::new();
    let mut updates = Vec::new();
    let mut deletes = Vec::new();
    let mut triggers = Vec::new();
    let mut returning_stars = Vec::new();
    let mut bounds = Vec::new();
    let mut mutation_column_uses = Vec::new();
    let mut insert_n = 0usize;
    let mut trigger_n = 0usize;
    let tokens = Tokens::new(sql);
    let table_index = collect_bounds.then(|| bounds::TableTokenIndex::new(tokens.all()));
    let mut out = FactOut {
        insert_n: &mut insert_n,
        trigger_n: &mut trigger_n,
        inserts: &mut inserts,
        selects: &mut selects,
        updates: &mut updates,
        deletes: &mut deletes,
        triggers: &mut triggers,
        returning_stars: &mut returning_stars,
        mutation_column_uses: &mut mutation_column_uses,
    };
    let mut temporary_relations = bounds::TemporaryRelations::default();
    for statement in statements {
        let scope = table_index
            .as_ref()
            .map(|index| bounds::Scope::with_table_tokens(index.cursor_at(statement.span().start)));
        let mut executed = Vec::new();
        wrappers::walk_executed(statement, &mut executed);
        for statement in executed {
            writes::collect(statement, &mut writes);
            collect_one(sql, statement, placeholder_positions, &mut out);
            if let Some(scope) = &scope {
                let first_bound = bounds.len();
                bounds::collect(statement, scope, placeholder_positions, &mut bounds);
                temporary_relations.apply(
                    statement,
                    &mut bounds[first_bound..],
                    scope,
                    placeholder_positions,
                );
            }
        }
    }
    dedupe::exists_set_operations(&mut selects);
    let (limit_uses, sweeps) = sweeps::collect(
        &tokens,
        statements,
        placeholder_positions.unwrap_or_default(),
        placeholder_positions.is_some(),
    );
    SqlStatementFileFacts {
        path: Default::default(),
        writes,
        inserts,
        selects,
        updates,
        deletes,
        triggers,
        returning_stars,
        mutation_column_uses,
        offset_uses: super::super::offset::offset_facts(sql, statements),
        bounds,
        limit_uses,
        sweeps,
        parse_failed,
        insert_keyword_count,
        has_top_level_not_exists: not_exists::has_top_level_conjunctive_not_exists(&masked),
        origin_line: 0,
    }
}

struct FactOut<'a> {
    insert_n: &'a mut usize,
    trigger_n: &'a mut usize,
    inserts: &'a mut Vec<SqlInsertFact>,
    selects: &'a mut Vec<SqlSelectFact>,
    updates: &'a mut Vec<Vec<SqlRelationPredicateFact>>,
    deletes: &'a mut Vec<Vec<SqlRelationPredicateFact>>,
    triggers: &'a mut Vec<SqlTriggerFact>,
    returning_stars: &'a mut Vec<SqlStarProjectionFact>,
    mutation_column_uses: &'a mut Vec<SqlColumnUseFact>,
}

fn collect_one(
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

pub fn has_top_level_not_exists_in(sql: &str) -> bool {
    not_exists::has_top_level_conjunctive_not_exists(&fallback::mask_quoted_sql(sql))
}
