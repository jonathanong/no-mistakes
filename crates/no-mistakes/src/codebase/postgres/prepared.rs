//! Request-owned SQL projections, prepared before rule dispatch.
use super::{EmbeddedSqlOptions, SqlSchemaFileFacts, SqlStatementFileFacts};
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::ts_source::SourceStore;
use rayon::prelude::*;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Debug)]
pub(crate) struct PreparationError {
    message: Arc<str>,
    source_kind: Option<std::io::ErrorKind>,
}
type Entry<T> = Result<Arc<T>, Arc<PreparationError>>;
mod fragments;
pub(crate) use fragments::PreparedSqlFragment;
#[derive(Clone, Default)]
pub(crate) struct PreparedPostgresFacts {
    pub schema: BTreeMap<PathBuf, Entry<SqlSchemaFileFacts>>,
    pub statements: BTreeMap<PathBuf, Entry<Vec<SqlStatementFileFacts>>>,
    pub embedded: BTreeMap<(PathBuf, EmbeddedSqlOptions), Entry<Vec<SqlStatementFileFacts>>>,
    pub fragments: BTreeMap<(PathBuf, EmbeddedSqlOptions), Entry<Vec<PreparedSqlFragment>>>,
}

pub(crate) fn prepare(
    root: &Path,
    files: &[PathBuf],
    sources: &SourceStore,
    plan: &CheckFactPlan,
    facts: &CheckFactMap,
) -> PreparedPostgresFacts {
    let mut out = PreparedPostgresFacts::default();
    if plan.postgres_schema || plan.postgres_dml {
        // Invalid globs are diagnosed when each rule validates/selects its paths.
        let paths = super::postgres_sql_paths(
            root,
            files,
            &super::PostgresSchemaOptions {
                sql_include: plan.postgres_sql_include.clone(),
            },
        )
        .unwrap_or_default();
        let rows: Vec<_> = paths
            .par_iter()
            .map(|path| {
                let source = sources.read_path(path).map_err(|error| {
                    Arc::new(PreparationError {
                        message: Arc::from(format!("failed to collect PostgreSQL facts: {error}")),
                        source_kind: Some(error.kind()),
                    })
                });
                let parsed = source.as_ref().map(|source| {
                    let parsed = super::parse::parse_postgres_sql(source);
                    let failed = parsed.is_err();
                    (
                        parsed.unwrap_or_else(|_| super::parse::parse_postgres_sql_lenient(source)),
                        failed,
                    )
                });
                let schema = parsed
                    .as_ref()
                    .map(|(statements, _)| {
                        let mut value = super::migration::extract_from_parsed(
                            source.as_ref().unwrap(),
                            statements,
                        );
                        value.path = path.clone();
                        Arc::new(value)
                    })
                    .map_err(|error| Arc::clone(error));
                let statements = parsed
                    .as_ref()
                    .map(|(statements, failed)| {
                        let mut value = super::statements::extract_from_parsed(
                            source.as_ref().unwrap(),
                            statements,
                            *failed,
                        );
                        value.path = path.clone();
                        Arc::new(vec![value])
                    })
                    .map_err(|error| Arc::clone(error));
                (path.clone(), schema, statements)
            })
            .collect();
        for (path, schema, statements) in rows {
            if plan.postgres_schema {
                out.schema.insert(path.clone(), schema);
            }
            if plan.postgres_dml {
                out.statements.insert(path, statements);
            }
        }
    }
    if plan.postgres_dml || plan.postgres_fragments {
        for path in files
            .iter()
            .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        {
            for profile in &plan.embedded_sql_options {
                let file = facts.embedded_sql(path, profile);
                if plan.postgres_fragments {
                    let entry = file
                        .as_ref()
                        .map(|file| Arc::new(fragments::collect(file)))
                        .map_err(|error| {
                            Arc::new(PreparationError {
                                message: Arc::from(format!(
                                    "failed to collect PostgreSQL facts: {error}"
                                )),
                                source_kind: None,
                            })
                        });
                    out.fragments.insert((path.clone(), profile.clone()), entry);
                }
                if !plan.postgres_dml {
                    continue;
                }
                let entry = file
                    .map(|file| Arc::new(super::collect::dml::embedded_call_facts(file)))
                    .map_err(|error| {
                        Arc::new(PreparationError {
                            message: Arc::from(format!(
                                "failed to collect PostgreSQL facts: {error}"
                            )),
                            source_kind: None,
                        })
                    });
                out.embedded.insert((path.clone(), profile.clone()), entry);
            }
        }
    }
    out
}

impl PreparedPostgresFacts {
    pub(crate) fn sql_source_not_found(&self, path: &Path) -> bool {
        matches!(self.statements.get(path), Some(Err(error))
            if error.source_kind == Some(std::io::ErrorKind::NotFound))
    }

    pub fn fragments(
        &self,
        path: &Path,
        profile: &EmbeddedSqlOptions,
    ) -> anyhow::Result<&[PreparedSqlFragment]> {
        entry(
            self.fragments.get(&(path.to_path_buf(), profile.clone())),
            path,
        )
        .map(|value| value.as_slice())
    }
    pub fn schema(&self, path: &Path) -> anyhow::Result<&SqlSchemaFileFacts> {
        let value = self.schema.get(path).or_else(|| {
            self.schema
                .get(&crate::codebase::ts_resolver::normalize_path(path))
        });
        entry(value, path).map(Arc::as_ref)
    }
    pub fn statements(
        &self,
        path: &Path,
        profile: Option<&EmbeddedSqlOptions>,
    ) -> anyhow::Result<&[SqlStatementFileFacts]> {
        let value = match profile {
            Some(profile) => self.embedded.get(&(path.to_path_buf(), profile.clone())),
            None => self.statements.get(path),
        };
        entry(value, path).map(|value| value.as_slice())
    }
}
fn entry<'a, T>(value: Option<&'a Entry<T>>, path: &Path) -> anyhow::Result<&'a Arc<T>> {
    match value {
        Some(Ok(value)) => Ok(value),
        Some(Err(error)) => Err(anyhow::anyhow!(error.message.to_string())),
        None => Err(anyhow::anyhow!(
            "prepared PostgreSQL facts are missing for {}",
            path.display()
        )),
    }
}

#[cfg(test)]
mod tests;
