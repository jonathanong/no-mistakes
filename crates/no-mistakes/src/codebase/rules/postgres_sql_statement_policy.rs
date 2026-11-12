use super::RuleFinding;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod options;
mod scan;
pub(crate) use options::Options;

pub const RULE_ID: &str = "postgres-sql-statement-policy";

const DEFAULT_BANNED: &[&str] = &[
    "CREATE TABLE",
    "ALTER TABLE",
    "CREATE INDEX",
    "CREATE VIEW",
    "TRUNCATE",
    "DROP INDEX",
    "DROP VIEW",
];

struct CompiledOptions {
    paths: super::RuleGlobFilter,
    schema: PostgresSchemaOptions,
    banned: HashSet<String>,
    settings: HashSet<String>,
    embedded: EmbeddedSqlOptions,
    fail_unanalyzable: bool,
    advise_unanalyzable: bool,
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
    sources: &std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        root,
        all_files,
        std::sync::Arc::clone(sources),
        config,
        &[RULE_ID],
    )?;
    let mut findings =
        check_with_files_sources_and_facts(root, config, all_files, sources, &facts)?;
    findings.extend(
        super::postgres_unanalyzable_schema::diagnostics_for_rule(
            root, config, all_files, &facts, RULE_ID,
        )?
        .findings,
    );
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
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
        let mut files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        files.retain(|path| compiled.paths.includes(&sql_rel(root, path)));
        let view = facts.for_sql_application(index, rule.name.as_deref());
        findings.extend(scan::scan(
            root,
            &compiled,
            &files,
            sources,
            view.as_ref()
                .map_or(facts as &dyn SqlFactContext, |view| view),
        )?);
    }
    Ok(findings.finish())
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let mut banned = HashSet::new();
    for value in &opts.banned_statements {
        let kind = value
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_ascii_uppercase();
        if kind == "DDL" {
            banned.extend(
                crate::codebase::postgres::SUPPORTED_KINDS
                    .iter()
                    .map(|kind| (*kind).to_string()),
            );
        } else if crate::codebase::postgres::SUPPORTED_KINDS.contains(&kind.as_str()) {
            banned.insert(kind);
        } else {
            anyhow::bail!("{RULE_ID}: unknown bannedStatements kind or group `{value}`");
        }
    }
    Ok(CompiledOptions {
        paths: super::RuleGlobFilter::new(RULE_ID, &opts.include, &opts.exclude)?,
        schema: PostgresSchemaOptions {
            sql_include: crate::codebase::postgres::effective_sql_include(
                &opts.sql_include,
                opts.sql_include_locked,
            ),
        },
        banned,
        settings: opts
            .banned_settings
            .iter()
            .map(|name| name.trim().to_ascii_lowercase())
            .collect(),
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_extra_executor_names(opts.extra_executor_names.as_deref())
            .with_exclude_executor_names(&opts.exclude_executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_trusted_sql_tags(&opts.trusted_sql_tags),
        fail_unanalyzable: crate::codebase::postgres::fail_unanalyzable_sql(
            RULE_ID,
            &opts.unanalyzable_sql,
        )?,
        advise_unanalyzable: opts.unanalyzable_sql.trim().eq_ignore_ascii_case("advise"),
    })
}

fn sql_rel(root: &Path, path: &Path) -> String {
    relative_slash_path(root, path)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod expanded_tests;

#[cfg(test)]
mod context_tests;

#[cfg(test)]
mod parse_failure_tests;
