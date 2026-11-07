use super::RuleFinding;
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
mod policy;
mod scan;
use policy::{Allow, Mode, WriteSet};
pub const RULE_ID: &str = "postgres-lock-written-rows-only";
pub(super) const CHECKS: [&str; 4] = [
    "row-lock-unwritten-relation",
    "table-lock",
    "constant-advisory-lock",
    "row-lock-outside-transaction",
];
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    include: Vec<String>,
    exclude: Vec<String>,
    import_specifier: String,
    executor_names: Vec<String>,
    executor_factory_names: Vec<String>,
    executor_type_names: Vec<String>,
    trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    unanalyzable_sql: String,
    checks: BTreeMap<String, Mode>,
    allow: Vec<Allow>,
    write_sets: Vec<WriteSet>,
}
struct CompiledOptions {
    paths: super::RuleGlobFilter,
    embedded: EmbeddedSqlOptions,
    unanalyzable: Mode,
    checks: BTreeMap<String, Mode>,
    allow: Vec<Allow>,
    write_sets: Vec<WriteSet>,
    allow_audit: Option<super::allow_audit::AllowAudit>,
    write_sets_audit: Option<super::allow_audit::AllowAudit>,
    incomplete_files: dashmap::DashSet<String>,
    checked_files: dashmap::DashSet<String>,
}
super::prepared_rule_entrypoints!(super::check_prepared_sql_rule);
pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    super::run_included_postgres_rule(
        super::PreparedRuleContext::new(root, config, all_files, sources, facts),
        RULE_ID,
        |rule| {
            let mut compiled = compile(&rule.try_rule_options()?)?;
            compiled.allow_audit = Some(super::allow_audit::AllowAudit::new(
                root,
                config,
                rule,
                Some(sources),
                "allow",
                compiled.allow.len(),
            ));
            compiled.write_sets_audit = Some(super::allow_audit::AllowAudit::new(
                root,
                config,
                rule,
                Some(sources),
                "writeSets",
                compiled.write_sets.len(),
            ));
            Ok(compiled)
        },
        |options, rel| options.paths.includes(rel),
        scan::scan,
    )
}
fn compile(options: &Options) -> Result<CompiledOptions> {
    policy::validate(options)?;
    let unanalyzable =
        match crate::codebase::postgres::unanalyzable_sql_mode(RULE_ID, &options.unanalyzable_sql)?
        {
            crate::codebase::postgres::UnanalyzableSqlMode::Fail => Mode::Fail,
            crate::codebase::postgres::UnanalyzableSqlMode::Advise => Mode::Advise,
            crate::codebase::postgres::UnanalyzableSqlMode::Ignore => Mode::Ignore,
        };
    Ok(CompiledOptions {
        paths: super::RuleGlobFilter::new(RULE_ID, &options.include, &options.exclude)?,
        embedded: EmbeddedSqlOptions::configured(
            &options.import_specifier,
            &options.executor_names,
        )
        .with_scoped_executors(
            &options.executor_factory_names,
            &options.executor_type_names,
        )
        .with_trusted_sql_tags(&options.trusted_sql_tags),
        incomplete_files: dashmap::DashSet::new(),
        checked_files: dashmap::DashSet::new(),
        allow_audit: None,
        write_sets_audit: None,
        unanalyzable,
        checks: options.checks.clone(),
        allow: options.allow.clone(),
        write_sets: options.write_sets.clone(),
    })
}
#[cfg(test)]
mod tests;
