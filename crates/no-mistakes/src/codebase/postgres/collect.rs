use super::embedded::{extract_embedded_sql_from_source, EmbeddedSqlFileFacts, EmbeddedSqlOptions};
use super::migration::extract_migration_facts;
use super::types::{PostgresFactError, PostgresFacts, PostgresSchemaOptions, SqlSchemaFileFacts};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::glob_normalize;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

mod dml;

/// Read `sql_paths` through `sources` and extract migration schema facts.
#[inline(never)]
pub fn extract_schema_facts(
    _root: &Path,
    sources: &SourceStore,
    sql_paths: &[PathBuf],
) -> Result<Vec<SqlSchemaFileFacts>, PostgresFactError> {
    let mut facts = sql_paths
        .par_iter()
        .map(|path| schema_file_facts(path, sources))
        .collect::<Result<Vec<_>, _>>()?;
    facts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(facts)
}

/// Filter candidates with `sql_include` globs, then extract schema facts.
#[inline(never)]
pub fn collect_schema_facts(
    root: &Path,
    sources: &SourceStore,
    candidate_paths: &[PathBuf],
    options: &PostgresSchemaOptions,
) -> Result<Vec<SqlSchemaFileFacts>, PostgresFactError> {
    let sql_paths = postgres_sql_paths(root, candidate_paths, options)?;
    extract_schema_facts(root, sources, &sql_paths)
}

/// Select static PostgreSQL files using the same `sqlInclude` semantics as
/// schema fact collection. Query rules use this when an opt-in needs to scan
/// checked-in `.sql` query files rather than only typed executor calls.
#[inline(never)]
pub fn postgres_sql_paths(
    root: &Path,
    candidate_paths: &[PathBuf],
    options: &PostgresSchemaOptions,
) -> Result<Vec<PathBuf>, PostgresFactError> {
    let globs = compile_sql_include(&options.sql_include)?;
    Ok(candidate_paths
        .iter()
        .filter(|path| matches_sql_include(root, path, &globs))
        .cloned()
        .collect())
}

/// Read TS/JS paths through `sources` and extract executor call SQL.
#[inline(never)]
pub fn extract_embedded_sql_facts(
    _root: &Path,
    sources: &SourceStore,
    ts_paths: &[PathBuf],
    options: &EmbeddedSqlOptions,
) -> Result<Vec<EmbeddedSqlFileFacts>, PostgresFactError> {
    let mut facts = ts_paths
        .par_iter()
        .map(|path| embedded_file_facts(path, sources, options))
        .collect::<Result<Vec<_>, _>>()?;
    facts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(facts)
}

/// Collect only the fact sets requested by `plan`. Later rules call this.
#[inline(never)]
pub fn collect_postgres_facts(
    root: &Path,
    sources: &SourceStore,
    files: &[PathBuf],
    plan: &CheckFactPlan,
    schema_options: &PostgresSchemaOptions,
    embedded_options: &EmbeddedSqlOptions,
) -> Result<PostgresFacts, PostgresFactError> {
    let schema = if plan.postgres_schema {
        collect_schema_facts(root, sources, files, schema_options)?
    } else {
        Vec::new()
    };
    let embedded = if plan.embedded_sql || plan.postgres_dml {
        let ts_paths: Vec<PathBuf> = files
            .iter()
            .filter(|path| is_indexable(path))
            .cloned()
            .collect();
        extract_embedded_sql_facts(root, sources, &ts_paths, embedded_options)?
    } else {
        Vec::new()
    };
    let statements = if plan.postgres_dml {
        dml::collect(root, sources, files, schema_options, &embedded)?
    } else {
        Vec::new()
    };
    Ok(PostgresFacts {
        schema,
        embedded,
        statements,
    })
}

#[inline(never)]
fn schema_file_facts(
    path: &Path,
    sources: &SourceStore,
) -> Result<SqlSchemaFileFacts, PostgresFactError> {
    let source = read_source(path, sources)?;
    let mut facts = extract_migration_facts(&source);
    facts.path = path.to_path_buf();
    Ok(facts)
}

#[inline(never)]
fn embedded_file_facts(
    path: &Path,
    sources: &SourceStore,
    options: &EmbeddedSqlOptions,
) -> Result<EmbeddedSqlFileFacts, PostgresFactError> {
    let source = read_source(path, sources)?;
    Ok(extract_embedded_sql_from_source(path, &source, options))
}

#[inline(never)]
pub(super) fn read_source(
    path: &Path,
    sources: &SourceStore,
) -> Result<std::sync::Arc<str>, PostgresFactError> {
    sources
        .read_path(path)
        .map_err(|error| PostgresFactError::for_path(path, format!("failed to read: {error}")))
}

#[inline(never)]
pub(super) fn compile_sql_include(patterns: &[String]) -> Result<GlobSet, PostgresFactError> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(&glob_normalize::normalize(pattern)).map_err(|error| {
            PostgresFactError::message(format!("invalid sqlInclude glob {pattern:?}: {error}"))
        })?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|error| PostgresFactError::message(format!("invalid sqlInclude globs: {error}")))
}

#[inline(never)]
pub(super) fn matches_sql_include(root: &Path, path: &Path, globs: &GlobSet) -> bool {
    let relative = relative_slash_path(root, path);
    globs.is_match(&relative)
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| globs.is_match(name))
}

#[cfg(test)]
mod tests;
