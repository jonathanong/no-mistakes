use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};
mod options;
pub(crate) use options::Options;
mod scan;
pub const RULE_ID: &str = "postgres-idempotent-insert";
pub(crate) struct CompiledOptions {
    include: GlobMatcher,
    exclude: GlobMatcher,
    schema: PostgresSchemaOptions,
    embedded: EmbeddedSqlOptions,
    fail_unanalyzable: bool,
    advise_unanalyzable: bool,
    scan_embedded: bool,
    check_convergence: bool,
    check_volatility: bool,
    check_arbiter: bool,
    check_triggers: bool,
    check_generated: bool,
    replay_safe: Vec<String>,
    trigger_writes: Vec<(String, Vec<String>)>,
}

impl CompiledOptions {
    fn includes(&self, rel: &str) -> bool {
        (self.include.is_empty() || self.include.is_match(rel))
            && (self.exclude.is_empty() || !self.exclude.is_match(rel))
    }
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    check_with_files_sources_and_optional_facts(root, config, all_files, sources, None)
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    check_with_files_sources_and_optional_facts(root, config, all_files, sources, Some(facts))
}

fn check_with_files_sources_and_optional_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    let mut findings = super::PostgresFindings::default();
    for (index, rule) in config.rule_applications(RULE_ID).into_iter().enumerate() {
        let opts: Options = rule.try_rule_options()?;
        let compiled = compile_options(&opts)?;
        let target_roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots))
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        let files: Vec<PathBuf> = files
            .into_iter()
            .filter(|path| compiled.includes(&relative_slash_path(root, path)))
            .collect();
        findings.extend(match facts {
            Some(facts) => {
                let view = facts.for_sql_application(index, rule.name.as_deref());
                scan::prepared(
                    root,
                    &compiled,
                    &files,
                    sources,
                    view.as_ref()
                        .map_or(facts as &dyn SqlFactContext, |view| view),
                )?
            }
            None => scan::scan(root, &compiled, &files, sources)?,
        });
    }
    Ok(findings.finish())
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
    let exclude = GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?;
    Ok(CompiledOptions {
        include,
        exclude,
        schema: PostgresSchemaOptions {
            sql_include: crate::codebase::postgres::effective_sql_include(
                &opts.sql_include,
                opts.sql_include_locked,
            ),
        },
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_trusted_sql_tags(&opts.trusted_sql_tags),
        fail_unanalyzable: crate::codebase::postgres::fail_unanalyzable_sql(
            RULE_ID,
            &opts.unanalyzable_sql,
        )?,
        advise_unanalyzable: opts.unanalyzable_sql.trim().eq_ignore_ascii_case("advise"),
        scan_embedded: opts.scan_embedded,
        check_convergence: opts.check_convergence,
        check_volatility: opts.check_volatility,
        check_arbiter: opts.check_arbiter,
        check_triggers: opts.check_triggers,
        check_generated: opts.check_generated,
        replay_safe: opts.replay_safe_trigger_functions.clone(),
        trigger_writes: opts
            .trigger_written_columns
            .iter()
            .map(|(name, columns)| (name.clone(), columns.clone()))
            .collect(),
    })
}

#[cfg(test)]
mod options_tests;
#[cfg(test)]
mod tests;
