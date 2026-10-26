use super::{compile_sql_include, matches_sql_include};
use crate::codebase::postgres::embedded::EmbeddedSqlFileFacts;
use crate::codebase::postgres::statement_facts::SqlLocationDemand;
use crate::codebase::postgres::statement_facts::SqlStatementFileFacts;
use crate::codebase::postgres::statements::extract_sql_statement_facts_with_json_writes;
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
    collect_json_writes: bool,
) -> Result<Vec<SqlStatementFileFacts>, PostgresFactError> {
    let globs = compile_sql_include(&schema_options.sql_include)?;
    let mut facts = files
        .par_iter()
        .filter(|path| matches_sql_include(root, path, &globs))
        .map(|path| {
            let source = match sources.read_path(path) {
                Ok(source) => source,
                Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
                    return Ok(super::encoding::statement_failure(path));
                }
                Err(error) => {
                    return Err(PostgresFactError::for_path(
                        path,
                        format!("failed to read: {error}"),
                    ))
                }
            };
            let mut file = extract_sql_statement_facts_with_json_writes(
                &source,
                collect_bounds,
                None,
                SqlLocationDemand::None,
                collect_json_writes,
            );
            file.path = path.clone();
            Ok(file)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for file in embedded {
        facts.extend(if collect_json_writes {
            embedded_call_facts_with_json_writes(
                file,
                collect_bounds,
                false,
                SqlLocationDemand::None,
                true,
            )
        } else {
            embedded_call_facts(file, collect_bounds, false)
        });
    }
    facts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(facts)
}

pub(crate) fn embedded_call_facts(
    file: &EmbeddedSqlFileFacts,
    collect_bounds: bool,
    variants_only: bool,
) -> Vec<SqlStatementFileFacts> {
    embedded_call_facts_with_locations(file, collect_bounds, variants_only, false)
}

pub(crate) fn embedded_call_facts_with_locations(
    file: &EmbeddedSqlFileFacts,
    collect_bounds: bool,
    variants_only: bool,
    exact_locations: bool,
) -> Vec<SqlStatementFileFacts> {
    embedded_call_facts_with_demand(
        file,
        collect_bounds,
        variants_only,
        if exact_locations {
            SqlLocationDemand::Full
        } else {
            SqlLocationDemand::None
        },
    )
}

pub(crate) fn embedded_call_facts_with_demand(
    file: &EmbeddedSqlFileFacts,
    collect_bounds: bool,
    variants_only: bool,
    demand: SqlLocationDemand,
) -> Vec<SqlStatementFileFacts> {
    embedded_call_facts_with_json_writes(file, collect_bounds, variants_only, demand, false)
}

pub(crate) fn embedded_call_facts_with_json_writes(
    file: &EmbeddedSqlFileFacts,
    collect_bounds: bool,
    variants_only: bool,
    demand: SqlLocationDemand,
    collect_json_writes: bool,
) -> Vec<SqlStatementFileFacts> {
    embedded::embedded_call_facts_with_json_writes(
        file,
        collect_bounds,
        variants_only,
        demand,
        collect_json_writes,
    )
}

mod embedded;

mod rebase;
#[cfg(test)]
use rebase::rebase_embedded_lines;

#[cfg(test)]
mod tests;
