use super::super::path_filter::GlobMatcher;
use crate::codebase::postgres::EmbeddedSqlOptions;
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) executor_factory_names: Vec<String>,
    pub(crate) executor_type_names: Vec<String>,
    pub(crate) trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
}

pub(crate) struct CompiledOptions {
    pub(crate) include: GlobMatcher,
    pub(crate) exclude: GlobMatcher,
    pub(crate) embedded: EmbeddedSqlOptions,
}

impl CompiledOptions {
    pub(crate) fn includes(&self, rel: &str) -> bool {
        let excluded = !self.exclude.is_empty() && self.exclude.is_match(rel);
        !excluded
            && crate::codebase::dependencies::extract::is_indexable(std::path::Path::new(rel))
            && (self.include.is_empty() || self.include.is_match(rel))
    }
}

pub(crate) fn compile(rule_id: &str, opts: &Options) -> Result<CompiledOptions> {
    Ok(CompiledOptions {
        include: GlobMatcher::new(&opts.include, &format!("{rule_id} include"))?,
        exclude: GlobMatcher::new(&opts.exclude, &format!("{rule_id} exclude"))?,
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_trusted_sql_tags(&opts.trusted_sql_tags),
    })
}

#[cfg(test)]
mod tests;
