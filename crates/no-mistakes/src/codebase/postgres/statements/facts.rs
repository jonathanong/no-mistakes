use super::tokens::Tokens;
use super::*;
mod collect;
mod inserts;
mod lifecycle;
mod locations;
mod parse;
mod policy;
use crate::codebase::postgres::parse::PreparedSql;
use collect::{collect_one, FactOut};
pub(crate) use lifecycle::project_bounds;
use lifecycle::LifecycleBuilder;
pub use parse::extract_sql_statement_facts;
pub(crate) use parse::{
    extract_from_parsed_with_recovered_placeholders, extract_sql_statement_facts_with_bounds,
    extract_sql_statement_facts_with_recovered_placeholders, extract_sql_variant_statement_facts,
};
use policy::table_keyword;
use sqlparser::ast::{Spanned, Statement};
use sqlparser::tokenizer::TokenWithSpan;
use std::collections::HashMap;
use std::sync::Arc;

mod prepared;
use prepared::PreparedStatements;

#[derive(Default)]
pub(crate) struct StatementPolicySources<'a> {
    pub(crate) schema: Option<&'a crate::codebase::postgres::SqlSchemaFileFacts>,
    pub(crate) functions: &'a [crate::codebase::postgres::SqlFunctionCallFact],
}

#[derive(Default)]
struct StatementSources<'a> {
    locations: bool,
    tokens: Option<&'a [Option<Arc<[TokenWithSpan]>>]>,
    policy: StatementPolicySources<'a>,
}

fn extract_from_parsed_and_sources(
    sql: &str,
    prepared_sql: &PreparedSql<'_>,
    statements: &[Statement],
    sources: StatementSources<'_>,
    parse_failed: bool,
    collect_bounds: bool,
    placeholder_positions: super::value::PlaceholderPositions<'_>,
) -> SqlStatementFileFacts {
    let masked = fallback::mask_quoted_sql(sql);
    let insert_keyword_count = fallback::insert_keyword_count(&masked);
    let mut writes = Vec::new();
    let mut locations = sources
        .locations
        .then(|| locations::Locations::new(placeholder_positions.unwrap_or_default()));
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
    let tokens = Tokens::with_prepared(sql, prepared_sql.tokens());
    let has_table_token = tokens.all().iter().any(table_keyword);
    let need_table_tokens = collect_bounds || has_table_token;
    let table_index = need_table_tokens.then(|| bounds::TableTokenIndex::new(tokens.all()));
    let mut recovered_indexes = HashMap::new();
    let insert_sources = super::lines::InsertSources::new(sql);
    let mut out = FactOut {
        insert_sources: &insert_sources,
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
    let mut lifecycle = LifecycleBuilder::default();
    for (index, source_statement) in statements.iter().enumerate() {
        let recovered_source = sources
            .tokens
            .and_then(|sources| sources.get(index))
            .and_then(Option::as_ref);
        let source_has_table =
            recovered_source.is_some_and(|source| source.iter().any(table_keyword));
        let source_index = recovered_source
            .filter(|_| collect_bounds || source_has_table)
            .map(|source| {
                recovered_indexes
                    .entry(Arc::as_ptr(source) as *const () as usize)
                    .or_insert_with(|| bounds::TableTokenIndex::new(source))
            });
        let table_tokens = source_index.map(|index| &*index).or(table_index.as_ref());
        let scope = collect_bounds
            .then(|| {
                table_tokens.map(|index| {
                    bounds::Scope::with_table_tokens(index.cursor_at(source_statement.span().start))
                })
            })
            .flatten();
        let mut table_cursor = (has_table_token || source_has_table)
            .then(|| table_tokens.map(|index| index.cursor_at(source_statement.span().start)))
            .flatten();
        let mut executed = Vec::new();
        wrappers::walk_executed(source_statement, &mut executed);
        for statement in executed {
            if let Some(locations) = &mut locations {
                locations.collect(statement);
            }
            writes::collect(statement, &mut writes);
            collect_one(
                sql,
                statement,
                placeholder_positions,
                table_cursor.as_mut(),
                &mut out,
            );
            if let Some(scope) = &scope {
                let statement_start =
                    table_tokens.and_then(|index| index.statement_start_at(statement.span().start));
                let (first_bound, view_reads) = lifecycle.collect(
                    statement,
                    statement_start,
                    scope,
                    placeholder_positions,
                    &temporary_relations,
                    &mut bounds,
                );
                prepared.apply(
                    source_statement,
                    statement,
                    &mut temporary_relations,
                    &mut bounds[first_bound..],
                    view_reads.as_ref(),
                    None,
                );
            }
        }
        if scope.is_some() {
            prepared.record(source_statement);
            lifecycle.finish_batch(source_statement);
        }
    }
    dedupe::exists_set_operations(&mut selects);
    let (limit_uses, sweeps) = sweeps::collect(
        &tokens,
        statements,
        placeholder_positions.unwrap_or_default(),
        placeholder_positions.is_some(),
    );
    let (statement_kinds, setting_uses, function_calls) =
        policy::collect(sql, prepared_sql, statements, sources.policy);
    let mut facts = SqlStatementFileFacts {
        variant_locations: None,
        path: Default::default(),
        statement_kinds,
        setting_uses,
        function_calls,
        writes,
        inserts,
        selects,
        updates,
        deletes,
        triggers,
        returning_stars,
        mutation_column_uses,
        offset_uses: super::super::offset::offset_facts_prepared(prepared_sql, statements),
        bounds,
        lifecycle: lifecycle.finish(),
        limit_uses,
        sweeps,
        parse_failed,
        insert_keyword_count,
        has_top_level_not_exists: not_exists::has_top_level_conjunctive_not_exists(&masked),
        origin_line: 0,
    };
    facts.variant_locations = locations.map(|locations| {
        locations::finish(
            locations,
            &facts,
            tokens.all(),
            sql,
            statements,
            placeholder_positions.unwrap_or_default(),
        )
    });
    facts
}

pub fn has_top_level_not_exists_in(sql: &str) -> bool {
    not_exists::has_top_level_conjunctive_not_exists(&fallback::mask_quoted_sql(sql))
}
