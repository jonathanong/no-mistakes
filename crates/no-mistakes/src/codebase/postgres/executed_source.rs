use super::{extract_embedded_sql_from_program, EmbeddedSqlKind, EmbeddedSqlOptions};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::ts_source::FileIdMap;
use anyhow::{bail, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

mod imports;
mod output;
use imports::{extract_source_with_imports, resolve_supplied_relative};
use output::{project_calls, statement};

/// Supplied TypeScript or JavaScript source; no repository read is performed.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutedPostgresSource {
    pub file_name: String,
    pub source: String,
}

/// An explicit import edge between two supplied source files.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutedPostgresResolvedImport {
    pub from_file_name: String,
    pub specifier: String,
    pub to_file_name: String,
}

/// One executor call or one complete alternative of a conditional executor call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutedPostgresSql {
    pub file_name: String,
    pub line: u32,
    pub annotation: Option<String>,
    pub sql_text: Option<String>,
    /// `dynamic` may expose only a verified prefix. Only complete SQL is safe
    /// for downstream statement classification.
    pub complete: bool,
    pub kind: EmbeddedSqlKind,
}

/// Reuse the embedded-SQL executor analysis for supplied source texts.
/// Ordering follows the supplied files, call sites, then concrete alternatives.
pub fn extract_executed_postgres_sql(
    sources: &[ExecutedPostgresSource],
    options: &EmbeddedSqlOptions,
) -> Result<Vec<ExecutedPostgresSql>> {
    extract_executed_postgres_sql_with_session(sources, options, &AnalysisSession::disabled())
}

/// Resolve imported SQL values and builder effects only among supplied files.
pub fn extract_executed_postgres_sql_with_imports(
    sources: &[ExecutedPostgresSource],
    options: &EmbeddedSqlOptions,
    imports: &[ExecutedPostgresResolvedImport],
) -> Result<Vec<ExecutedPostgresSql>> {
    extract_executed_postgres_sql_with_imports_and_session(
        sources,
        options,
        imports,
        &AnalysisSession::disabled(),
    )
}

pub(crate) fn extract_executed_postgres_sql_with_session(
    sources: &[ExecutedPostgresSource],
    options: &EmbeddedSqlOptions,
    session: &AnalysisSession,
) -> Result<Vec<ExecutedPostgresSql>> {
    let batches: Vec<_> = sources
        .par_iter()
        .map(|source| {
            crate::ast::with_owned_request_parse_cache(|| {
                let path = Path::new(&source.file_name);
                session.with_program(path, &Arc::from(source.source.as_str()), |program, text| {
                    project_calls(
                        &source.file_name,
                        extract_embedded_sql_from_program(path, program, text, options).calls,
                    )
                })
            })
        })
        .collect();
    let mut statements = Vec::new();
    for batch in batches {
        statements.extend(batch?);
    }
    Ok(statements)
}

pub(crate) fn extract_executed_postgres_sql_with_imports_and_session(
    sources: &[ExecutedPostgresSource],
    options: &EmbeddedSqlOptions,
    imports: &[ExecutedPostgresResolvedImport],
    session: &AnalysisSession,
) -> Result<Vec<ExecutedPostgresSql>> {
    let batches: Vec<_> = sources
        .par_iter()
        .map(|source| {
            crate::ast::with_owned_request_parse_cache(|| {
                extract_source_with_imports(source, options, session)
            })
        })
        .collect();
    let entries = batches.into_iter().collect::<Result<Vec<_>>>()?;
    let supplied = entries
        .iter()
        .map(|(path, _)| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<crate::fx::PathSet>();
    if supplied.len() != entries.len() {
        bail!("resolveImports requires distinct supplied fileName values");
    }
    let mut resolved = crate::fx::FxHashMap::default();
    for import in imports {
        let from = crate::codebase::ts_resolver::normalize_path(Path::new(&import.from_file_name));
        let to = crate::codebase::ts_resolver::normalize_path(Path::new(&import.to_file_name));
        if !supplied.contains(&from) || !supplied.contains(&to) {
            bail!("resolvedImports edge must name supplied source files");
        }
        let key = (from, import.specifier.clone());
        if resolved.insert(key, to).is_some() {
            bail!("duplicate resolvedImports edge");
        }
    }
    let mut files = FileIdMap::from_entries(entries);
    let projected_calls =
        super::query_annotation::project::project_executed(&mut files, |specifier, from| {
            if let Some(target) = resolved.get(&(from.to_path_buf(), specifier.to_string())) {
                return Some(target.clone());
            }
            if !specifier.starts_with('.') {
                return None;
            }
            resolve_supplied_relative(from, specifier, &supplied)
        });
    let mut statements = Vec::new();
    for source in sources {
        let facts = files
            .get(Path::new(&source.file_name))
            .expect("supplied source");
        let embedded = &facts.embedded_sql[0].1;
        for (index, call) in embedded.calls.iter().enumerate() {
            let mut statements_for_call = project_calls(&source.file_name, vec![call.clone()]);
            if call.kind == EmbeddedSqlKind::Dynamic || call.kind == EmbeddedSqlKind::Composed {
                let key = (
                    crate::codebase::ts_resolver::normalize_path(Path::new(&source.file_name)),
                    embedded.call_spans[index],
                );
                if let Some(projected) = projected_calls.get(&key) {
                    if let Some(alternatives) =
                        projected.as_ref().filter(|values| !values.is_empty())
                    {
                        statements_for_call = alternatives
                            .iter()
                            .map(|candidate| {
                                statement(
                                    &source.file_name,
                                    call.line,
                                    Some(candidate.text.clone()),
                                    candidate.complete,
                                    EmbeddedSqlKind::Composed,
                                )
                            })
                            .collect();
                    } else {
                        // A semantic execution with an unproven mutable
                        // effect cannot inherit a complete lexical result.
                        statements_for_call = vec![statement(
                            &source.file_name,
                            call.line,
                            None,
                            false,
                            EmbeddedSqlKind::Dynamic,
                        )];
                    }
                }
            }
            statements.extend(statements_for_call);
        }
    }
    Ok(statements)
}
