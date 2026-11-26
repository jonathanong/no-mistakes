use super::{baseline, DEFAULT_BASELINE_FILE, RULE_ID};
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::postgres::{
    compile_sql_include, effective_sql_include, embedded::ExecutorOption, matches_sql_include,
    EmbeddedSqlOptions, PostgresSchemaOptions, TrustedSqlTag,
};
use crate::codebase::rules::RuleGlobFilter;
use anyhow::{ensure, Result};
use globset::GlobSet;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::ops::Deref;
use std::path::Path;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct LockScanInput {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) sql_include: Vec<String>,
    pub(crate) sql_include_locked: bool,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) extra_executor_names: Option<Vec<String>>,
    pub(crate) exclude_executor_names: Vec<String>,
    pub(crate) executor_factory_names: Vec<String>,
    pub(crate) executor_type_names: Vec<String>,
    pub(crate) executor_option_properties: Vec<String>,
    pub(crate) executor_options: Vec<ExecutorOption>,
    pub(crate) trusted_sql_tags: Vec<TrustedSqlTag>,
    pub(crate) helpers: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum LockScanTarget {
    Sql,
    TypeScript,
}

pub(crate) struct LockScanOptions {
    pub(crate) paths: RuleGlobFilter,
    pub(crate) schema: PostgresSchemaOptions,
    pub(crate) embedded: EmbeddedSqlOptions,
    helpers: BTreeSet<String>,
    folded_sql_helpers: BTreeSet<String>,
}

impl LockScanOptions {
    pub(crate) fn compile(rule_id: &str, input: LockScanInput) -> Result<Self> {
        let helpers: BTreeSet<_> = input.helpers.into_iter().collect();
        ensure!(
            helpers.iter().all(|name| !name.trim().is_empty()),
            "{rule_id} helpers must be non-empty names"
        );
        let folded_sql_helpers = helpers
            .iter()
            .map(|name| name.to_ascii_lowercase())
            .collect();
        let mut embedded =
            EmbeddedSqlOptions::configured(&input.import_specifier, &input.executor_names)
                .with_extra_executor_names(input.extra_executor_names.as_deref())
                .with_exclude_executor_names(&input.exclude_executor_names)
                .with_scoped_executors(&input.executor_factory_names, &input.executor_type_names)
                .with_executor_option_properties(&input.executor_option_properties)
                .with_executor_options(&input.executor_options)
                .with_trusted_sql_tags(&input.trusted_sql_tags)
                .with_exact_constant_origins();
        embedded.track_transaction_helpers = true;
        let schema = PostgresSchemaOptions {
            sql_include: effective_sql_include(&input.sql_include, input.sql_include_locked),
        };
        compile_sql_include(&schema.sql_include)?;
        Ok(Self {
            paths: RuleGlobFilter::new(rule_id, &input.include, &input.exclude)?,
            schema,
            embedded,
            helpers,
            folded_sql_helpers,
        })
    }

    pub(crate) fn has_helpers(&self) -> bool {
        !self.helpers.is_empty()
    }

    pub(crate) fn matches_helper(&self, name: &str, terminal: &str) -> bool {
        self.helpers.contains(name) || self.helpers.contains(terminal)
    }

    pub(crate) fn matches_sql_helper(&self, name: &str, terminal: &str) -> bool {
        self.matches_helper(name, terminal)
            || self.folded_sql_helpers.contains(name)
            || self.folded_sql_helpers.contains(terminal)
    }

    pub(crate) fn scans_executors(&self) -> bool {
        self.embedded.selects_executors() || !self.embedded.trusted_sql_tags.is_empty()
    }

    pub(crate) fn classify_file(
        &self,
        root: &Path,
        path: &Path,
        sql: &GlobSet,
    ) -> Option<LockScanTarget> {
        if path.extension().is_some_and(|ext| ext == "sql") && matches_sql_include(root, path, sql)
        {
            Some(LockScanTarget::Sql)
        } else if is_indexable(path) && (self.has_helpers() || self.scans_executors()) {
            Some(LockScanTarget::TypeScript)
        } else {
            None
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(in crate::codebase::rules) struct Options {
    #[serde(flatten)]
    scan: LockScanInput,
    #[serde(alias = "baseline")]
    baseline_file: Option<String>,
    baseline_unanalyzed: bool,
}

pub(in crate::codebase::rules) struct CompiledOptions {
    scan: LockScanOptions,
    pub(super) baseline_file: String,
    pub(super) baseline_unanalyzed: bool,
}

impl Deref for CompiledOptions {
    type Target = LockScanOptions;

    fn deref(&self) -> &Self::Target {
        &self.scan
    }
}

impl CompiledOptions {
    pub(super) fn from_rule(rule: &crate::config::v2::schema::RuleDef) -> Result<Self> {
        Self::from_options(rule.try_rule_options()?)
    }

    pub(in crate::codebase::rules) fn from_options(opts: Options) -> Result<Self> {
        let baseline_file = baseline::relative_path(
            opts.baseline_file
                .as_deref()
                .unwrap_or(DEFAULT_BASELINE_FILE),
        )?;
        Ok(Self {
            scan: LockScanOptions::compile(RULE_ID, opts.scan)?,
            baseline_file: baseline_file.to_string_lossy().replace('\\', "/"),
            baseline_unanalyzed: opts.baseline_unanalyzed,
        })
    }
}
