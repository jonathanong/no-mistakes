use super::*;

/// Record the prepared executor profiles and the rules that selected each one.
pub fn assign_prepared_embedded_sql_options(
    config: &NoMistakesConfig,
    plan: &mut crate::codebase::check_facts::CheckFactPlan,
) -> Result<()> {
    let (options, annotation, rule_ids) = super::configured_embedded_sql_options_with_annotation(
        config,
        super::PREPARED_EMBEDDED_SQL_RULE_IDS,
    )?;
    plan.embedded_sql = !options.is_empty();
    plan.embedded_sql_head_calls_only = head_calls_only_profiles(&options, &annotation, &rule_ids);
    plan.embedded_sql_options = options;
    plan.embedded_sql_rule_ids = rule_ids;
    Ok(())
}

pub(super) fn head_calls_only_profiles(
    options: &[EmbeddedSqlOptions],
    annotation: &[EmbeddedSqlOptions],
    scans: &super::scans::EmbeddedSqlRuleScans,
) -> Vec<EmbeddedSqlOptions> {
    options
        .iter()
        .filter(|profile| {
            !annotation.contains(profile)
                && scans.get(*profile).is_some_and(|scans| {
                    !scans.is_empty()
                        && scans
                            .iter()
                            .all(|scan| scan.rule_id == "postgres-require-query-annotation")
                })
        })
        .cloned()
        .collect()
}

/// Declare SQL projections before the shared request fact pass.
pub fn configure_prepared_postgres_plan(
    root: &Path,
    config: &NoMistakesConfig,
    plan: &mut crate::codebase::check_facts::CheckFactPlan,
) -> Result<()> {
    plan.record_general_sql_locations();
    for (index, rule) in config.rules.iter().enumerate().filter(|(_, rule)| {
        rule.enabled
            && [
                "postgres-no-unbounded-sql-identifier",
                "postgres-no-mutable-sql-registry",
            ]
            .contains(&rule.rule.as_str())
            && config.rule_has_effective_target(rule)
    }) {
        if config.effective_rule_analysis(rule)?.effort == crate::config::v2::Effort::Low {
            let options: EmbeddedSqlRuleOptions = rule.try_rule_options()?;
            let profile = EmbeddedSqlOptions::configured(
                options.import_specifier.as_deref().unwrap_or_default(),
                &options.executor_names,
            )
            .with_scoped_executors(
                &options.executor_factory_names,
                &options.executor_type_names,
            )
            .with_trusted_sql_tags(&options.trusted_sql_tags);
            let scan =
                crate::codebase::postgres::sql_identifier_syntax::plan::SqlIdentifierScan::new(
                    root,
                    config,
                    rule,
                    index,
                    &options.include,
                    &options.exclude,
                )?;
            plan.sql_identifier_profiles
                .entry(profile)
                .or_default()
                .push(scan);
        }
    }
    let (_, annotation_options, _) = super::configured_embedded_sql_options_with_annotation(
        config,
        &["postgres-require-query-annotation"],
    )?;
    plan.query_annotation_heads |= super::annotation_effort::low_requested(config)?;
    let evaluated_annotation = !annotation_options.is_empty();
    if evaluated_annotation {
        if plan.query_annotation {
            if let Some(existing) = &mut plan.query_annotation_options {
                existing.extend(annotation_options);
                existing.sort();
                existing.dedup();
            }
        } else {
            plan.query_annotation_options = Some(annotation_options);
        }
        plan.query_annotation = true;
    }
    plan.postgres_variants |= evaluated_annotation
        || ["postgres-lock-ordering", "postgres-conflict-ordering"]
            .iter()
            .any(|id| !config.rule_applications(id).is_empty());
    let ordering = ["postgres-lock-ordering", "postgres-conflict-ordering"]
        .iter()
        .any(|id| !config.rule_applications(id).is_empty());
    plan.postgres_dml |= ordering;
    plan.postgres_exact_locations |= !config
        .rule_applications("postgres-lock-ordering")
        .is_empty();
    plan.postgres_conflict_locations |= !config
        .rule_applications("postgres-conflict-ordering")
        .is_empty();
    plan.postgres_general_locations |= super::PREPARED_EMBEDDED_SQL_RULE_IDS
        .iter()
        .any(|id| *id != "postgres-conflict-ordering" && !config.rule_applications(id).is_empty());
    let dml_rules = [
        "postgres-idempotent-insert",
        "postgres-required-predicates",
        "postgres-no-offset",
        "postgres-generated-column-predicates",
        "postgres-explicit-columns",
        "postgres-bounded-statements",
        "postgres-sql-shape-policy",
        "postgres-no-generated-column-writes",
    ];
    plan.postgres_dml |= dml_rules
        .iter()
        .any(|id| !config.rule_applications(id).is_empty())
        || !configured_embedded_sql_options(config, &["postgres-sql-statement-policy"])?.is_empty();
    plan.postgres_bounds |= !config
        .rule_applications("postgres-bounded-statements")
        .is_empty();
    plan.postgres_fragments |= !config
        .rule_applications("postgres-sql-shape-policy")
        .is_empty();
    let schema_rules = [
        "postgres-idempotent-insert",
        "postgres-generated-column-predicates",
        "postgres-no-generated-column-writes",
        "postgres-identifier-length",
        "postgres-no-add-column",
        "postgres-require-named-constraints",
        "postgres-redundant-index",
        "postgres-fk-index",
        "postgres-constraint-validate",
        "postgres-sql-statement-policy",
        "postgres-require-fk-on-delete",
    ];
    plan.postgres_schema |= schema_rules
        .iter()
        .any(|id| !config.rule_applications(id).is_empty());
    plan.postgres_sql_include
        .extend(sql_patterns(config, &schema_rules)?);
    plan.postgres_sql_include
        .extend(sql_patterns(config, &dml_rules)?);
    plan.postgres_write_sql_include
        .extend(write_patterns(config)?);
    plan.postgres_write_sql_include.sort();
    plan.postgres_write_sql_include.dedup();
    plan.postgres_sql_include.sort();
    plan.postgres_sql_include.dedup();
    plan.source |= [
        "postgres-no-mutable-sql-registry",
        "postgres-no-unbounded-sql-identifier",
    ]
    .iter()
    .any(|id| {
        config.rule_applications(id).iter().any(|rule| {
            config.effective_rule_analysis(rule).map_or(true, |policy| {
                policy.effort != crate::config::v2::Effort::Low
            })
        })
    });
    plan.raw_source |= !plan.sql_identifier_profiles.is_empty();
    Ok(())
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct WriteOptions {
    include: Vec<String>,
}

pub(super) fn write_patterns(config: &NoMistakesConfig) -> Result<Vec<String>> {
    let mut patterns = Vec::new();
    for rule in config.rule_applications("postgres-no-generated-column-writes") {
        let options: WriteOptions = rule.try_rule_options()?;
        patterns.extend(if options.include.is_empty() {
            vec!["**/*.sql".into()]
        } else {
            options.include
        });
    }
    Ok(patterns)
}
