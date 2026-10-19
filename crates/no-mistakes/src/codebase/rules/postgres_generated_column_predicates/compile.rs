use super::{validate, CompiledOptions, Options, RULE_ID};
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};
use anyhow::{bail, Result};

pub(super) fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let functions = match &opts.functions {
        None => vec!["uuid_extract_timestamp".to_string()],
        Some(values) if values.is_empty() => {
            bail!("{RULE_ID} option functions: must not be empty")
        }
        Some(values) => validate::unique_names(values, "functions", "function")?,
    };
    let (where_clause, join_clause, order_clause) = validate::clauses(opts.clauses.as_deref())?;
    Ok(CompiledOptions {
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
        functions,
        require_primary_key: opts.require_argument_is_primary_key.unwrap_or(true),
        where_clause,
        join_clause,
        order_clause,
        extras: validate::extras(&opts.extra_generated_columns)?,
    })
}
