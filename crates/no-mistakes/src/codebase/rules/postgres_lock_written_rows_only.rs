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
pub(crate) const SHARE_LOCK_THEN_WRITE: &str = "share-lock-then-write";
pub(crate) const REDUNDANT_PRELOCK: &str = "redundant-prelock";
pub(super) const CHECKS: [&str; 6] = [
    "row-lock-unwritten-relation",
    "table-lock",
    "constant-advisory-lock",
    "row-lock-outside-transaction",
    SHARE_LOCK_THEN_WRITE,
    REDUNDANT_PRELOCK,
];

pub(crate) fn is_sequence_check(check: &str) -> bool {
    matches!(check, SHARE_LOCK_THEN_WRITE | REDUNDANT_PRELOCK)
}

impl CompiledOptions {
    fn mode(&self, check: &str) -> Mode {
        self.checks.get(check).copied().unwrap_or_else(|| {
            if is_sequence_check(check) {
                Mode::Ignore
            } else {
                Mode::default()
            }
        })
    }
}
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    schema_catalog_path: String,
    include: Vec<String>,
    exclude: Vec<String>,
    import_specifier: String,
    executor_names: Vec<String>,
    extra_executor_names: Option<Vec<String>>,
    exclude_executor_names: Vec<String>,
    executor_factory_names: Vec<String>,
    executor_type_names: Vec<String>,
    transaction_executor_type_names: Vec<String>,
    executor_option_properties: Vec<String>,
    executor_options: Vec<crate::codebase::postgres::embedded::ExecutorOption>,
    trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    unanalyzable_sql: String,
    checks: BTreeMap<String, Mode>,
    allow: Vec<Allow>,
    write_sets: Vec<WriteSet>,
}
struct CompiledOptions {
    schema_catalog_path: Option<String>,
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
            let filter = super::path_filter::RulePathFilter::new(root, config, rule)?;
            compiled.schema_catalog_path = compiled
                .schema_catalog_path
                .filter(|path| filter.is_match(Path::new(path)));
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
    let mut embedded =
        EmbeddedSqlOptions::configured(&options.import_specifier, &options.executor_names)
            .with_extra_executor_names(options.extra_executor_names.as_deref())
            .with_exclude_executor_names(&options.exclude_executor_names)
            .with_scoped_executors(
                &options.executor_factory_names,
                &options.executor_type_names,
            )
            .with_transaction_executor_types(&options.transaction_executor_type_names)
            .with_executor_option_properties(&options.executor_option_properties)
            .with_executor_options(&options.executor_options)
            .with_trusted_sql_tags(&options.trusted_sql_tags);
    embedded.track_transaction_helpers = true;
    embedded.track_share_lock_writes = options
        .checks
        .get(SHARE_LOCK_THEN_WRITE)
        .is_some_and(|mode| *mode != Mode::Ignore);
    embedded.track_redundant_prelocks =
        options.checks.get(REDUNDANT_PRELOCK) == Some(&Mode::Advise);
    Ok(CompiledOptions {
        schema_catalog_path: (!options.schema_catalog_path.is_empty())
            .then(|| options.schema_catalog_path.clone()),
        paths: super::RuleGlobFilter::new(RULE_ID, &options.include, &options.exclude)?,
        embedded,
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
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
