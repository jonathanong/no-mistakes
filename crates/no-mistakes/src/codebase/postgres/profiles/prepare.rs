use super::*;

pub(crate) fn prepare_rule_sql_facts(
    root: &Path,
    files: &[PathBuf],
    sources: Arc<crate::codebase::ts_source::SourceStore>,
    config: &NoMistakesConfig,
    rule_ids: &[&str],
) -> Result<crate::codebase::check_facts::CheckFactMap> {
    let dml_ids: Vec<_> = rule_ids
        .iter()
        .copied()
        .filter(|id| PREPARED_EMBEDDED_SQL_RULE_IDS.contains(id))
        .collect();
    let profiles = configured_embedded_sql_options(config, &dml_ids)?;
    let patterns = sql_patterns(config, rule_ids)?;
    let selected = if dml_ids.is_empty() {
        crate::codebase::postgres::postgres_sql_paths(
            root,
            files,
            &crate::codebase::postgres::PostgresSchemaOptions {
                sql_include: patterns.clone(),
            },
        )
        .map_err(|error| anyhow::anyhow!("{} option sqlInclude: {error}", rule_ids.join(", ")))?
    } else {
        files.to_vec()
    };
    Ok(
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            root,
            selected,
            Vec::new(),
            crate::codebase::check_facts::CheckFactPlan {
                postgres_schema: rule_ids.iter().any(|id| *id != "postgres-sql-shape-policy"),
                postgres_dml: !dml_ids.is_empty(),
                postgres_fragments: rule_ids.contains(&"postgres-sql-shape-policy"),
                postgres_sql_include: patterns,
                embedded_sql: !profiles.is_empty(),
                embedded_sql_options: profiles,
                postgres_schema_catalog_paths: configured_schema_catalog_paths(config, rule_ids)?,
                ..Default::default()
            },
            None,
            sources,
        ),
    )
}
