use super::CheckFileFacts;
mod declared;
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::codebase::ts_source::FileIdMap;
pub(crate) use declared::{planned_low_applications, with_failed_source};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) type FailedSqlSources = BTreeSet<PathBuf>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SqlProjection {
    Embedded,
    AnnotationMax,
    AnnotationHeads,
    LowSyntax,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SqlSourceKey {
    pub projection: SqlProjection,
    pub profile: EmbeddedSqlOptions,
    pub application: Option<(String, usize)>,
}

impl SqlSourceKey {
    pub fn profile(projection: SqlProjection, profile: &EmbeddedSqlOptions) -> Self {
        Self {
            projection,
            profile: profile.clone(),
            application: None,
        }
    }
    pub fn application(
        projection: SqlProjection,
        profile: &EmbeddedSqlOptions,
        rule: &str,
        index: usize,
    ) -> Self {
        Self {
            projection,
            profile: profile.clone(),
            application: Some((rule.to_string(), index)),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SqlSourcePrerequisites {
    failures: BTreeMap<SqlSourceKey, BTreeMap<(u32, u32), FailedSqlSources>>,
    declared: Option<BTreeSet<SqlSourceKey>>,
}

impl SqlSourcePrerequisites {
    pub fn record(
        &mut self,
        key: &SqlSourceKey,
        span: (u32, u32),
        failures: impl IntoIterator<Item = PathBuf>,
    ) {
        let failures: FailedSqlSources = failures.into_iter().collect();
        if !failures.is_empty() {
            self.failures
                .entry(key.clone())
                .or_default()
                .entry(span)
                .or_default()
                .extend(failures);
        }
    }
    pub fn merge(&mut self, other: Self) {
        for (key, spans) in other.failures {
            for (span, failures) in spans {
                self.record(&key, span, failures);
            }
        }
    }
    pub fn failures<'a>(&'a self, key: &SqlSourceKey) -> impl Iterator<Item = &'a PathBuf> {
        self.failures
            .get(key)
            .into_iter()
            .flat_map(|spans| spans.values())
            .flat_map(|paths| paths.iter())
    }
    pub fn is_empty(&self) -> bool {
        self.failures.is_empty()
    }
}

pub(crate) fn source_error(file: &CheckFileFacts) -> Option<&str> {
    file.parse_error
        .as_deref()
        .or(file.ts.parse_error.as_deref())
        .or(file.ts.operational_error.as_deref())
        .or(file
            .ts
            .fatal_parse_error
            .then_some("prepared TypeScript source has a fatal parse error"))
}

pub(crate) fn source_failed(files: &FileIdMap<CheckFileFacts>, path: &Path) -> bool {
    files
        .get(path)
        .is_some_and(|file| source_error(file).is_some())
}

pub(crate) fn source_demanded(
    files: &FileIdMap<CheckFileFacts>,
    path: &Path,
    key: &SqlSourceKey,
) -> bool {
    if !matches!(
        key.projection,
        SqlProjection::LowSyntax | SqlProjection::AnnotationMax
    ) {
        return true;
    }
    let Some(file) = files.get(path) else {
        return true;
    };
    if let Some(declared) = &file.sql_source_prerequisites.declared {
        return declared.contains(key);
    }
    match key.projection {
        SqlProjection::LowSyntax if !file.sql_identifier_syntax.low_applications.is_empty() => {
            key.application.as_ref().is_some_and(|(rule, index)| {
                file.sql_identifier_syntax.low_applications.iter().any(
                    |(profile, owner, ordinal)| {
                        profile == &key.profile && owner == rule && ordinal == index
                    },
                )
            })
        }
        SqlProjection::AnnotationMax if !file.query_annotation.is_empty() => file
            .query_annotation
            .iter()
            .any(|(profile, _)| profile == &key.profile),
        _ => true,
    }
}

#[cfg(test)]
mod tests;
