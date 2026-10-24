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
    let (profiles, annotation_profiles, embedded_sql_rule_ids) =
        configured_embedded_sql_options_with_annotation(config, &dml_ids)?;
    let evaluated_annotation = !annotation_profiles.is_empty();
    let head_calls_only = super::plan::head_calls_only_profiles(
        &profiles,
        &annotation_profiles,
        &embedded_sql_rule_ids,
    );
    let low_annotation = rule_ids.contains(&"postgres-require-query-annotation")
        && super::annotation_effort::low_requested(config)?;
    let patterns = sql_patterns(config, rule_ids)?;
    let low_ids = [
        "postgres-no-offset",
        "postgres-no-transformed-uuid-predicates",
    ];
    let low_sql = low_ids.iter().any(|id| {
        rule_ids.contains(id)
            && config.rule_applications(id).iter().any(|rule| {
                config
                    .effective_rule_analysis(rule)
                    .is_ok_and(|policy| policy.effort == crate::config::v2::Effort::Low)
            })
    });
    let selected = if profiles.is_empty() && !low_sql {
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
    let mut low_plan = crate::codebase::check_facts::CheckFactPlan::default();
    if low_sql {
        let mut scoped = config.clone();
        scoped
            .rules
            .retain(|rule| low_ids.contains(&rule.rule.as_str()));
        super::plan::configure_prepared_postgres_plan(root, &scoped, &mut low_plan)?;
    }
    Ok(
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            root,
            selected,
            Vec::new(),
            crate::codebase::check_facts::CheckFactPlan {
                raw_source: low_plan.raw_source,
                sql_identifier_profiles: low_plan.sql_identifier_profiles,
                postgres_schema: rule_ids.iter().any(|id| {
                    !matches!(
                        *id,
                        "postgres-sql-shape-policy"
                            | "postgres-require-query-annotation"
                            | "postgres-lock-ordering"
                            | "postgres-conflict-ordering"
                    )
                }),
                postgres_dml: low_sql
                    || (!profiles.is_empty()
                        && dml_ids
                            .iter()
                            .any(|id| !matches!(*id, "postgres-require-query-annotation"))),
                postgres_variants: evaluated_annotation
                    || dml_ids.iter().any(|id| {
                        matches!(*id, "postgres-lock-ordering" | "postgres-conflict-ordering")
                    }),
                postgres_exact_locations: rule_ids.contains(&"postgres-lock-ordering"),
                postgres_conflict_locations: rule_ids.contains(&"postgres-conflict-ordering"),
                postgres_general_locations: rule_ids.iter().any(|id| {
                    *id != "postgres-conflict-ordering"
                        && super::PREPARED_EMBEDDED_SQL_RULE_IDS.contains(id)
                }),
                postgres_bounds: rule_ids.contains(&"postgres-bounded-statements"),
                postgres_fragments: rule_ids.contains(&"postgres-sql-shape-policy"),
                postgres_sql_include: patterns,
                postgres_write_sql_include: if rule_ids
                    .contains(&"postgres-no-generated-column-writes")
                {
                    super::plan::write_patterns(config)?
                } else {
                    Vec::new()
                },
                query_annotation: evaluated_annotation,
                query_annotation_heads: low_annotation,
                query_annotation_options: rule_ids
                    .contains(&"postgres-require-query-annotation")
                    .then_some(annotation_profiles),
                embedded_sql: !profiles.is_empty(),
                embedded_sql_options: profiles,
                embedded_sql_head_calls_only: head_calls_only,
                embedded_sql_rule_ids,
                postgres_schema_catalog_paths: configured_schema_catalog_paths(config, rule_ids)?,
                ..Default::default()
            },
            None,
            sources,
        ),
    )
}
