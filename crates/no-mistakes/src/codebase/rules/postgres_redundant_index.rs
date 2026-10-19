use super::RuleFinding;
use crate::codebase::postgres::PostgresSchemaOptions;
use crate::codebase::ts_source::relative_slash_path;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;

mod order;
mod redundancy;
mod scan;

pub const RULE_ID: &str = "postgres-redundant-index";
const DEFAULT_ALLOW_DIRECTIVE: &str = "redundant-index-allow";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) sql_include: Vec<String>,
    #[serde(default)]
    pub(crate) sql_include_locked: bool,
    pub(crate) allow_directive: String,
    pub(crate) allowed_indexes: Vec<String>,
}

struct CompiledOptions {
    schema: PostgresSchemaOptions,
    allow_directive: String,
    allowed_indexes: BTreeSet<String>,
}

super::prepared_rule_entrypoints!(super::check_prepared_sql_rule);

super::prepared_file_rule_facts!();

fn compile_options(opts: &Options) -> CompiledOptions {
    CompiledOptions {
        schema: super::schema_options(&opts.sql_include, opts.sql_include_locked),
        allow_directive: if opts.allow_directive.is_empty() {
            DEFAULT_ALLOW_DIRECTIVE.to_string()
        } else {
            opts.allow_directive.clone()
        },
        allowed_indexes: opts.allowed_indexes.iter().cloned().collect(),
    }
}

fn sql_rel(root: &Path, path: &Path) -> String {
    relative_slash_path(root, path)
}

#[cfg(test)]
mod tests;
