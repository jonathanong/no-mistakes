use super::RuleFinding;
use crate::codebase::postgres::PostgresSchemaOptions;
use crate::codebase::ts_source::relative_slash_path;
use serde::Deserialize;
use std::path::Path;

mod scan;

pub const RULE_ID: &str = "postgres-no-add-column";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) sql_include: Vec<String>,
    #[serde(default)]
    pub(crate) sql_include_locked: bool,
    pub(crate) allowed_migrations: Vec<AllowedMigration>,
}

#[derive(Clone, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AllowedMigration {
    pub(crate) path: String,
    pub(crate) table: String,
    pub(crate) column: String,
    #[serde(rename = "type")]
    pub(crate) data_type: String,
    pub(crate) nullable: bool,
    pub(crate) default: Option<String>,
}

struct CompiledOptions {
    schema: PostgresSchemaOptions,
    allowed_migrations: Vec<AllowedMigration>,
    missing: Vec<RuleFinding>,
}

super::prepared_rule_entrypoints!(super::check_prepared_sql_rule);

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    all_files: &[std::path::PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> anyhow::Result<Vec<RuleFinding>> {
    super::run_prepared_file_rule(
        super::PreparedRuleContext::new(root, config, all_files, sources, facts),
        RULE_ID,
        |rule| {
            let opts: Options = rule.try_rule_options()?;
            let mut compiled = compile_options(&opts);
            let selection = std::sync::OnceLock::new();
            compiled.missing = super::allow_audit::missing_file_findings(
                root,
                config,
                rule,
                sources,
                "allowedMigrations",
                opts.allowed_migrations.iter().map(|entry| {
                    let globs = selection.get_or_init(|| {
                        crate::codebase::postgres::compile_sql_include(&compiled.schema.sql_include)
                    });
                    let selected = globs.as_ref().is_ok_and(|globs| {
                        crate::codebase::postgres::matches_sql_include(
                            root,
                            &root.join(&entry.path),
                            globs,
                        )
                    });
                    (
                        if selected {
                            entry.path.clone()
                        } else {
                            String::new()
                        },
                        entry.path.clone(),
                    )
                }),
            );
            Ok(compiled)
        },
        scan::scan,
    )
}

fn compile_options(opts: &Options) -> CompiledOptions {
    CompiledOptions {
        schema: super::schema_options(&opts.sql_include, opts.sql_include_locked),
        allowed_migrations: opts.allowed_migrations.clone(),
        missing: Vec::new(),
    }
}

fn sql_rel(root: &Path, path: &Path) -> String {
    relative_slash_path(root, path)
}

#[cfg(test)]
mod tests;
