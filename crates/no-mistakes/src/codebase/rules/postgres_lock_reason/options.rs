use super::RULE_ID;
use crate::codebase::postgres::{embedded::ExecutorOption, TrustedSqlTag};
use crate::codebase::rules::postgres_lock_inventory::options::{LockScanInput, LockScanOptions};
use crate::config::v2::{schema::RuleDef, NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use serde::Deserialize;
use std::ops::Deref;

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    include: Vec<String>,
    exclude: Vec<String>,
    sql_include: Vec<String>,
    sql_include_locked: bool,
    import_specifier: String,
    executor_names: Vec<String>,
    extra_executor_names: Option<Vec<String>>,
    exclude_executor_names: Vec<String>,
    executor_factory_names: Vec<String>,
    executor_type_names: Vec<String>,
    executor_option_properties: Vec<String>,
    executor_options: Vec<ExecutorOption>,
    trusted_sql_tags: Vec<TrustedSqlTag>,
    helpers: Vec<String>,
    schema_catalog_path: Option<String>,
    report_unmatched_executor_names: bool,
    unanalyzable_sql: Option<String>,
}

pub(super) struct CompiledOptions {
    scan: LockScanOptions,
}

impl Deref for CompiledOptions {
    type Target = LockScanOptions;

    fn deref(&self) -> &Self::Target {
        &self.scan
    }
}

impl CompiledOptions {
    pub(super) fn from_rule(rule: &RuleDef) -> Result<Self> {
        let opts: Options = rule.try_rule_options()?;
        let _ = (
            &opts.schema_catalog_path,
            opts.report_unmatched_executor_names,
            &opts.unanalyzable_sql,
        );
        let scan = LockScanOptions::compile(
            RULE_ID,
            LockScanInput {
                include: opts.include,
                exclude: opts.exclude,
                sql_include: opts.sql_include,
                sql_include_locked: opts.sql_include_locked,
                import_specifier: opts.import_specifier,
                executor_names: opts.executor_names,
                extra_executor_names: opts.extra_executor_names,
                exclude_executor_names: opts.exclude_executor_names,
                executor_factory_names: opts.executor_factory_names,
                executor_type_names: opts.executor_type_names,
                executor_option_properties: opts.executor_option_properties,
                executor_options: opts.executor_options,
                trusted_sql_tags: opts.trusted_sql_tags,
                helpers: opts.helpers,
            },
        )?;
        Ok(Self { scan })
    }
}

pub(super) fn resolve_unanalyzable(
    config: &NoMistakesConfig,
    rule: &RuleDef,
) -> Result<Unanalyzable> {
    crate::config::v2::validate_low_alias(rule)?;
    let explicit = rule.options.get("unanalyzable").is_some()
        || rule.options.get("unanalyzableSql").is_some()
        || config.unanalyzable.is_some()
        || config.unanalyzable_sql.is_some();
    if explicit {
        Ok(config.effective_rule_analysis(rule)?.unanalyzable)
    } else {
        Ok(Unanalyzable::Advise)
    }
}
