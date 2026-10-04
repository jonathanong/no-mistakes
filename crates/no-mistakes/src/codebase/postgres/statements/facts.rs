use super::tokens::Tokens;
use super::*;
mod collect;
mod inserts;
use crate::codebase::postgres::parse::{
    parse_postgres_sql, parse_postgres_sql_lenient_with_sources,
};
use collect::{collect_one, FactOut};
use sqlparser::ast::{Spanned, Statement};
use sqlparser::tokenizer::TokenWithSpan;
use std::collections::HashMap;
use std::sync::Arc;

mod prepared;
use prepared::PreparedStatements;

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
    match parsed {
        Ok(statements) => extract_from_parsed_with_recovered_placeholders(
            sql,
            &statements,
            false,
            collect_bounds,
            placeholder_positions,
        ),
        Err(_) => {
            let (statements, sources): (Vec<_>, Vec<_>) =
                parse_postgres_sql_lenient_with_sources(sql)
                    .into_iter()
                    .map(|located| (located.statement, located.source))
                    .unzip();
            extract_from_parsed_and_sources(
                sql,
                &statements,
                Some(&sources),
                parse_failed,
                collect_bounds,
                placeholder_positions,
            )
        }
    }
}

pub(crate) fn extract_from_parsed_with_recovered_placeholders(
    sql: &str,
    statements: &[Statement],
    parse_failed: bool,
    collect_bounds: bool,
    placeholder_positions: super::value::PlaceholderPositions<'_>,
) -> SqlStatementFileFacts {
    extract_from_parsed_and_sources(
        sql,
        statements,
        None,
        parse_failed,
        collect_bounds,
        placeholder_positions,
    )
}

fn extract_from_parsed_and_sources(
    sql: &str,
    statements: &[Statement],
    sources: Option<&[Option<Arc<[TokenWithSpan]>>]>,
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
    let mut recovered_indexes = HashMap::new();
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
    let mut prepared = PreparedStatements::default();
    for (index, source_statement) in statements.iter().enumerate() {
        let recovered_source = collect_bounds
            .then(|| sources.and_then(|sources| sources.get(index)))
            .flatten()
            .and_then(Option::as_ref);
        let source_index = recovered_source.map(|source| {
            recovered_indexes
                .entry(Arc::as_ptr(source) as *const () as usize)
                .or_insert_with(|| bounds::TableTokenIndex::new(source))
        });
        let scope = source_index
            .map(|index| &*index)
            .or(table_index.as_ref())
            .map(|index| {
                bounds::Scope::with_table_tokens(index.cursor_at(source_statement.span().start))
            });
        let mut executed = Vec::new();
        wrappers::walk_executed(source_statement, &mut executed);
        for statement in executed {
            writes::collect(statement, &mut writes);
            collect_one(sql, statement, placeholder_positions, &mut out);
            if let Some(scope) = &scope {
                let first_bound = bounds.len();
                bounds::collect(statement, scope, placeholder_positions, &mut bounds);
                prepared.apply(
                    source_statement,
                    statement,
                    &mut temporary_relations,
                    &mut bounds[first_bound..],
                    scope,
                    placeholder_positions,
                );
            }
        }
        if scope.is_some() {
            prepared.record(source_statement);
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

pub fn has_top_level_not_exists_in(sql: &str) -> bool {
    not_exists::has_top_level_conjunctive_not_exists(&fallback::mask_quoted_sql(sql))
}
