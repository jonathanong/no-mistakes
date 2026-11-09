use super::*;
pub(super) fn options(opts: &Options) -> Result<CompiledOptions> {
    GlobMatcher::new(&opts.sql_include, &format!("{RULE_ID} sqlInclude"))?;
    Ok(CompiledOptions {
        schema_catalog_path: opts.schema_catalog_path.clone(),
        include: GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?,
        exclude: GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?,
        schema: PostgresSchemaOptions {
            sql_include: crate::codebase::postgres::effective_sql_include(
                &opts.sql_include,
                opts.sql_include_locked,
            ),
        },
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_extra_executor_names(opts.extra_executor_names.as_deref())
            .with_exclude_executor_names(&opts.exclude_executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_executor_option_properties(&opts.executor_option_properties)
            .with_trusted_sql_tags(&opts.trusted_sql_tags)
            .with_sql_literal_helpers(&opts.sql_literal_helpers),
    })
}
