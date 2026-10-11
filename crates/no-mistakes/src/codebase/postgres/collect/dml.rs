use super::{compile_sql_include, matches_sql_include, read_source};
use crate::codebase::postgres::embedded::{EmbeddedSqlFileFacts, EmbeddedSqlKind};
use crate::codebase::postgres::statement_facts::SqlStatementFileFacts;
use crate::codebase::postgres::statements::{
    extract_sql_statement_facts_with_bounds,
    extract_sql_statement_facts_with_recovered_placeholders, extract_sql_variant_statement_facts,
};
use crate::codebase::postgres::types::{PostgresFactError, PostgresSchemaOptions};
use crate::codebase::ts_source::SourceStore;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub(super) fn collect(
    root: &Path,
    sources: &SourceStore,
    files: &[PathBuf],
    schema_options: &PostgresSchemaOptions,
    embedded: &[EmbeddedSqlFileFacts],
    collect_bounds: bool,
) -> Result<Vec<SqlStatementFileFacts>, PostgresFactError> {
    let globs = compile_sql_include(&schema_options.sql_include)?;
    let mut facts = files
        .par_iter()
        .filter(|path| matches_sql_include(root, path, &globs))
        .map(|path| {
            let source = read_source(path, sources)?;
            let mut file = extract_sql_statement_facts_with_bounds(&source, collect_bounds);
            file.path = path.clone();
            Ok(file)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for file in embedded {
        facts.extend(embedded_call_facts(file, collect_bounds, false));
    }
    facts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(facts)
}

pub(crate) fn embedded_call_facts(
    file: &EmbeddedSqlFileFacts,
    collect_bounds: bool,
    variants_only: bool,
) -> Vec<SqlStatementFileFacts> {
    file.calls
        .iter()
        .enumerate()
        .filter(|(_, call)| !variants_only || !call.variants.is_empty())
        .flat_map(|(call_index, call)| {
            call.statement_calls()
                .enumerate()
                .map(move |(variant_index, version)| {
                    (
                        version,
                        !call.variants.is_empty(),
                        call_index,
                        variant_index,
                    )
                })
        })
        .filter_map(|(call, variant, call_index, variant_index)| {
            let sql = call.sql_text.as_deref()?;
            let extract = if variant {
                extract_sql_variant_statement_facts
            } else {
                extract_sql_statement_facts_with_recovered_placeholders
            };
            let mut facts = extract(sql, collect_bounds, &call.recovered_placeholder_positions);
            if call.kind == EmbeddedSqlKind::Dynamic {
                // Recovered interpolation text can prove OFFSET syntax and write
                // targets. Other rules keep their existing dynamic-SQL failure policy.
                facts = SqlStatementFileFacts {
                    offset_uses: facts.offset_uses,
                    writes: facts.writes,
                    ..Default::default()
                };
            }
            facts.path = file.path.clone();
            if let Some(locations) = &mut facts.variant_locations {
                locations.original_call_line = call.line;
                locations.call_index = call_index;
                locations.variant_index = variant_index;
            }
            rebase_embedded_lines(&mut facts, &call, variant);
            Some(facts)
        })
        .collect()
}

mod rebase;
use rebase::rebase_embedded_lines;

#[cfg(test)]
mod tests;
