use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod functions;
mod iteration;
mod scan;

pub const RULE_ID: &str = "postgres-sql-shape-policy";

mod shapes;
use shapes::*;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) sql_include: Vec<String>,
    #[serde(default)]
    pub(crate) sql_include_locked: bool,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) executor_factory_names: Vec<String>,
    pub(crate) executor_type_names: Vec<String>,
    pub(crate) trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    pub(crate) banned_shapes: Vec<String>,
    pub(crate) unanalyzable_sql: String,
    pub(crate) shape_options: iteration::ShapeOptions,
}

pub(crate) struct CompiledOptions {
    include: GlobMatcher,
    exclude: GlobMatcher,
    schema: PostgresSchemaOptions,
    embedded: EmbeddedSqlOptions,
    fail_unanalyzable: bool,
    advise_unanalyzable: bool,
    shapes: BannedShapes,
    iteration: iteration::IterationOptions,
    banned_functions: Vec<functions::CompiledFunction>,
}

impl CompiledOptions {
    fn includes(&self, rel: &str) -> bool {
        (self.include.is_empty() || self.include.is_match(rel))
            && (self.exclude.is_empty() || !self.exclude.is_match(rel))
    }
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
        |rule| compile_options(&rule.try_rule_options()?),
        |compiled, rel| compiled.includes(rel),
        scan::scan,
    )
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
    let exclude = GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?;
    let shapes = banned_shapes(&opts.banned_shapes)?;
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
        shapes,
        banned_functions: functions::compile(
            &opts.shape_options.banned_function_call,
            shapes.banned_function_call,
        )?,
        iteration: iteration::compile(&opts.shape_options)?,
    })
}

#[cfg(test)]
mod select_all_tests;
#[cfg(test)]
mod shape_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod followup_tests;

#[cfg(test)]
mod implicit_fetch_tests;

#[cfg(test)]
mod review_fix_tests;

#[cfg(test)]
mod merge_tests;

#[cfg(test)]
mod prepared_tests;

#[cfg(test)]
mod table_sweep_tests;

#[cfg(test)]
mod function_tests;

#[cfg(test)]
mod function_projection_tests;

#[cfg(test)]
mod clause_tests;

mod select_findings;
