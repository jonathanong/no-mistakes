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
    pub(crate) extra_executor_names: Option<Vec<String>>,
    pub(crate) exclude_executor_names: Vec<String>,
    pub(crate) executor_factory_names: Vec<String>,
    pub(crate) executor_type_names: Vec<String>,
    pub(crate) trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    pub(crate) finite_domain_assertions: Vec<FiniteDomainAssertion>,
    pub(crate) identifier_validators: Vec<IdentifierValidator>,
    pub(crate) identifier_sources: Vec<IdentifierSource>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FiniteDomainAssertion {
    pub(crate) name: String,
    pub(crate) argument: usize,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IdentifierValidator {
    pub(crate) module: Option<String>,
    pub(crate) name: String,
    pub(crate) argument: usize,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IdentifierSource {
    pub(crate) module: Option<String>,
    pub(crate) name: String,
}

pub(crate) struct CompiledOptions {
    pub(crate) include: GlobMatcher,
    pub(crate) exclude: GlobMatcher,
    pub(crate) embedded: EmbeddedSqlOptions,
    pub(crate) finite_domain_assertions: Vec<FiniteDomainAssertion>,
    pub(crate) identifier_validators: Vec<IdentifierValidator>,
    pub(crate) identifier_sources: Vec<IdentifierSource>,
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
            .with_extra_executor_names(opts.extra_executor_names.as_deref())
            .with_exclude_executor_names(&opts.exclude_executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_trusted_sql_tags(&opts.trusted_sql_tags),
        finite_domain_assertions: opts.finite_domain_assertions.clone(),
        identifier_validators: opts.identifier_validators.clone(),
        identifier_sources: opts.identifier_sources.clone(),
    })
}

#[cfg(test)]
mod tests;
