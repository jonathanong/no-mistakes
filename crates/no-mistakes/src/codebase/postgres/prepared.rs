//! Request-owned SQL projections, prepared before rule dispatch.
use super::{EmbeddedSqlOptions, SqlSchemaFileFacts, SqlStatementFileFacts};
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::ts_source::SourceStore;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

mod sql;

#[derive(Clone, Debug)]
pub(crate) struct PreparationError {
    pub(crate) message: Arc<str>,
    pub(crate) source_kind: Option<std::io::ErrorKind>,
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
        sql::collect(root, files, sources, plan, &mut out);
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
    pub(crate) fn readable_schema(
        &self,
        path: &Path,
    ) -> anyhow::Result<Option<&SqlSchemaFileFacts>> {
        let value = self.schema.get(path).or_else(|| {
            self.schema
                .get(&crate::codebase::ts_resolver::normalize_path(path))
        });
        Ok(known_entry(value, path)?.as_ref().ok().map(Arc::as_ref))
    }
    pub(crate) fn readable_statements(
        &self,
        path: &Path,
        profile: Option<&EmbeddedSqlOptions>,
    ) -> anyhow::Result<Option<&[SqlStatementFileFacts]>> {
        let value = match profile {
            Some(profile) => self.embedded.get(&(path.to_path_buf(), profile.clone())),
            None => self.statements.get(path),
        };
        Ok(known_entry(value, path)?
            .as_ref()
            .ok()
            .map(|value| value.as_slice()))
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
fn known_entry<'a, T>(value: Option<&'a Entry<T>>, path: &Path) -> anyhow::Result<&'a Entry<T>> {
    value.ok_or_else(|| {
        anyhow::anyhow!(
            "prepared PostgreSQL facts are missing for {}",
            path.display()
        )
    })
}
fn entry<'a, T>(value: Option<&'a Entry<T>>, path: &Path) -> anyhow::Result<&'a Arc<T>> {
    known_entry(value, path)?
        .as_ref()
        .map_err(|error| anyhow::anyhow!(error.message.to_string()))
}

#[cfg(test)]
mod tests;

pub(crate) fn write_sql_paths(
    root: &Path,
    files: &[PathBuf],
    plan: &CheckFactPlan,
) -> Vec<PathBuf> {
    let candidates: Vec<_> = files
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sql"))
        .cloned()
        .collect();
    super::postgres_sql_paths(
        root,
        &candidates,
        &super::PostgresSchemaOptions {
            sql_include: plan.postgres_write_sql_include.clone(),
        },
    )
    .unwrap_or_default()
}
