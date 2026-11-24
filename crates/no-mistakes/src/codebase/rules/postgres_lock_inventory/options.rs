use super::{baseline, DEFAULT_BASELINE_FILE, RULE_ID};
use crate::codebase::postgres::{
    effective_sql_include, embedded::ExecutorOption, EmbeddedSqlOptions, PostgresSchemaOptions,
    TrustedSqlTag,
};
use crate::codebase::rules::RuleGlobFilter;
use anyhow::{ensure, Result};
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
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
    #[serde(alias = "baseline")]
    baseline_file: Option<String>,
    helpers: Vec<String>,
}

pub(super) struct CompiledOptions {
    pub(super) paths: RuleGlobFilter,
    pub(super) schema: PostgresSchemaOptions,
    pub(super) embedded: EmbeddedSqlOptions,
    pub(super) baseline_file: String,
    helpers: BTreeSet<String>,
    folded_sql_helpers: BTreeSet<String>,
}

impl CompiledOptions {
    pub(super) fn from_rule(rule: &crate::config::v2::schema::RuleDef) -> Result<Self> {
        let opts: Options = rule.try_rule_options()?;
        let baseline_file = baseline::relative_path(
            opts.baseline_file
                .as_deref()
                .unwrap_or(DEFAULT_BASELINE_FILE),
        )?;
        let helpers: BTreeSet<_> = opts.helpers.into_iter().collect();
        ensure!(
            helpers.iter().all(|name| !name.trim().is_empty()),
            "{RULE_ID} helpers must be non-empty names"
        );
        let folded_sql_helpers = helpers
            .iter()
            .map(|name| name.to_ascii_lowercase())
            .collect();
        let mut embedded =
            EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
                .with_extra_executor_names(opts.extra_executor_names.as_deref())
                .with_exclude_executor_names(&opts.exclude_executor_names)
                .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
                .with_executor_option_properties(&opts.executor_option_properties)
                .with_executor_options(&opts.executor_options)
                .with_trusted_sql_tags(&opts.trusted_sql_tags)
                .with_exact_constant_origins();
        embedded.track_transaction_helpers = true;
        Ok(Self {
            paths: RuleGlobFilter::new(RULE_ID, &opts.include, &opts.exclude)?,
            schema: PostgresSchemaOptions {
                sql_include: effective_sql_include(&opts.sql_include, opts.sql_include_locked),
            },
            embedded,
            baseline_file: baseline_file.to_string_lossy().replace('\\', "/"),
            helpers,
            folded_sql_helpers,
        })
    }

    pub(super) fn has_helpers(&self) -> bool {
        !self.helpers.is_empty()
    }

    pub(super) fn matches_helper(&self, name: &str, terminal: &str) -> bool {
        self.helpers.contains(name) || self.helpers.contains(terminal)
    }

    pub(super) fn matches_sql_helper(&self, name: &str, terminal: &str) -> bool {
        self.matches_helper(name, terminal)
            || self.folded_sql_helpers.contains(name)
            || self.folded_sql_helpers.contains(terminal)
    }

    pub(super) fn scans_executors(&self) -> bool {
        self.embedded.selects_executors() || !self.embedded.trusted_sql_tags.is_empty()
    }
}
