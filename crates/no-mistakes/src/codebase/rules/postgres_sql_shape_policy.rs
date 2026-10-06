use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod iteration;
mod scan;

pub const RULE_ID: &str = "postgres-sql-shape-policy";

const CORRELATED_EXISTS_SET_OP: &str = "correlated-exists-set-operation";
const NOT_IN_SUBQUERY: &str = "not-in-subquery";
const COUNT_FOR_EXISTENCE: &str = "count-for-existence";
const LITERAL_LIMIT: &str = "literal-limit";
const KEYSET_ONLY_SWEEP: &str = "keyset-only-sweep";

#[derive(Clone, Copy, Default)]
pub(crate) struct BannedShapes {
    correlated_exists_set_operation: bool,
    not_in_subquery: bool,
    count_for_existence: bool,
    literal_limit: bool,
    keyset_only_sweep: bool,
}

impl BannedShapes {
    fn unanalyzable_target(&self) -> &'static str {
        if self.correlated_exists_set_operation {
            CORRELATED_EXISTS_SET_OP
        } else if self.not_in_subquery {
            NOT_IN_SUBQUERY
        } else if self.count_for_existence {
            COUNT_FOR_EXISTENCE
        } else if self.literal_limit {
            LITERAL_LIMIT
        } else {
            KEYSET_ONLY_SWEEP
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) sql_include: Vec<String>,
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
    shapes: BannedShapes,
    iteration: iteration::IterationOptions,
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
    sources: &std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        root,
        all_files,
        std::sync::Arc::clone(sources),
        config,
        &[RULE_ID],
    )?;
    check_with_files_sources_and_facts(root, config, all_files, &facts)
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.try_rule_options()?;
            let compiled = compile_options(&opts)?;
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|path| {
                    super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
                })
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            let files: Vec<PathBuf> = files
                .into_iter()
                .filter(|path| compiled.includes(&relative_slash_path(root, path)))
                .collect();
            scan::scan(root, &compiled, &files, facts)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
    let exclude = GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?;
    let shapes = banned_shapes(&opts.banned_shapes)?;
    Ok(CompiledOptions {
        include,
        exclude,
        schema: PostgresSchemaOptions {
            sql_include: if opts.sql_include.is_empty() {
                PostgresSchemaOptions::default().sql_include
            } else {
                opts.sql_include.clone()
            },
        },
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_trusted_sql_tags(&opts.trusted_sql_tags),
        fail_unanalyzable: crate::codebase::postgres::fail_unanalyzable_sql(
            RULE_ID,
            &opts.unanalyzable_sql,
        )?,
        shapes,
        iteration: iteration::compile(&opts.shape_options)?,
    })
}

fn banned_shapes(values: &[String]) -> Result<BannedShapes> {
    let mut shapes = BannedShapes::default();
    let values = if values.is_empty() {
        vec![CORRELATED_EXISTS_SET_OP.to_string()]
    } else {
        values.to_vec()
    };
    for shape in &values {
        if shape.eq_ignore_ascii_case(CORRELATED_EXISTS_SET_OP) {
            shapes.correlated_exists_set_operation = true;
        } else if shape.eq_ignore_ascii_case(NOT_IN_SUBQUERY) {
            shapes.not_in_subquery = true;
        } else if shape.eq_ignore_ascii_case(COUNT_FOR_EXISTENCE) {
            shapes.count_for_existence = true;
        } else if shape.eq_ignore_ascii_case(LITERAL_LIMIT) {
            shapes.literal_limit = true;
        } else if shape.eq_ignore_ascii_case(KEYSET_ONLY_SWEEP) {
            shapes.keyset_only_sweep = true;
        } else {
            anyhow::bail!("{RULE_ID}: unknown bannedShapes value `{shape}`");
        }
    }
    Ok(shapes)
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
