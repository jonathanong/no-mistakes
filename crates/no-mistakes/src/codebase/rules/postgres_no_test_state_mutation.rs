//! Prevent persistent database mutations in explicitly selected test code.
mod scan;

use super::{path_filter::GlobMatcher, RuleFinding};
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const RULE_ID: &str = "postgres-no-test-state-mutation";

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    test_globs: Vec<String>,
    helper_roots: Vec<String>,
    import_specifier: String,
    executor_names: Vec<String>,
    executor_factory_names: Vec<String>,
    executor_type_names: Vec<String>,
    trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    unanalyzable_sql: String,
}

struct CompiledOptions {
    tests: GlobMatcher,
    helpers: GlobMatcher,
    embedded: EmbeddedSqlOptions,
    fail_unanalyzable: bool,
    advise_unanalyzable: bool,
}

impl CompiledOptions {
    fn new(options: &Options) -> Result<Self> {
        if options.test_globs.is_empty() {
            bail!("{RULE_ID}: testGlobs must list at least one test-file glob");
        }
        Ok(Self {
            tests: GlobMatcher::new(&options.test_globs, &format!("{RULE_ID} testGlobs"))?,
            helpers: GlobMatcher::new(&options.helper_roots, &format!("{RULE_ID} helperRoots"))?,
            embedded: EmbeddedSqlOptions::configured(
                &options.import_specifier,
                &options.executor_names,
            )
            .with_scoped_executors(
                &options.executor_factory_names,
                &options.executor_type_names,
            )
            .with_trusted_sql_tags(&options.trusted_sql_tags),
            fail_unanalyzable: crate::codebase::postgres::fail_unanalyzable_sql(
                RULE_ID,
                &options.unanalyzable_sql,
            )?,
            advise_unanalyzable: options
                .unanalyzable_sql
                .trim()
                .eq_ignore_ascii_case("advise"),
        })
    }

    fn includes(&self, root: &Path, path: &Path) -> bool {
        let rel = relative_slash_path(root, path);
        crate::codebase::dependencies::extract::is_indexable(path)
            && (self.tests.is_match(&rel) || self.helpers.is_match(&rel))
    }
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(files);
    check_with_files_and_sources(root, config, files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        root,
        files,
        Arc::clone(sources),
        config,
        &[RULE_ID],
    )?;
    check_with_files_sources_and_facts(root, config, files, sources, &facts)
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let mut findings = super::PostgresFindings::default();
    for (index, rule) in config.rule_applications(RULE_ID).into_iter().enumerate() {
        let options: Options = rule.try_rule_options()?;
        let compiled = CompiledOptions::new(&options)?;
        let files = super::selected_rule_files(root, config, rule, all_files)?;
        let files: Vec<_> = files
            .into_iter()
            .filter(|path| compiled.includes(root, path))
            .collect();
        let view = facts.for_sql_application(index, rule.name.as_deref());
        findings.extend(scan::scan(
            root,
            &compiled,
            &files,
            sources,
            view.as_ref().map_or(
                facts as &dyn crate::codebase::check_facts::SqlFactContext,
                |view| view,
            ),
        )?);
    }
    Ok(findings.finish())
}

#[cfg(test)]
mod tests;
