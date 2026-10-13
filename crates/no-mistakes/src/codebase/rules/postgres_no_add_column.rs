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
}

super::prepared_rule_entrypoints!(super::check_prepared_sql_rule);

super::prepared_file_rule_facts!();

fn compile_options(opts: &Options) -> CompiledOptions {
    CompiledOptions {
        schema: super::schema_options_with_fallback(&opts.sql_include),
        allowed_migrations: opts.allowed_migrations.clone(),
    }
}

fn sql_rel(root: &Path, path: &Path) -> String {
    relative_slash_path(root, path)
}

#[cfg(test)]
mod tests;
