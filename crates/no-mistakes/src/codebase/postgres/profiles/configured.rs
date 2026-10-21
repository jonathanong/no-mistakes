use super::*;

type AnnotatedSqlProfiles = (
    Vec<EmbeddedSqlOptions>,
    Vec<EmbeddedSqlOptions>,
    scans::EmbeddedSqlRuleScans,
);

pub(super) fn configured_embedded_sql_options_with_annotation(
    config: &NoMistakesConfig,
    rule_ids: &[&str],
) -> Result<AnnotatedSqlProfiles> {
    configured_with_annotation_masks(config, rule_ids, |_, _| {})
}

pub(super) fn configured_with_annotation_masks<'a>(
    config: &'a NoMistakesConfig,
    rule_ids: &[&str],
    mut visit: impl FnMut(
        &'a crate::config::v2::schema::RuleDef,
        Arc<crate::codebase::rules::RuleGlobFilter>,
    ),
) -> Result<AnnotatedSqlProfiles> {
    let mut grouped = scans::EmbeddedSqlRuleScans::new();
    let mut annotation_profiles = Vec::new();
    for rule_id in rule_ids {
        for rule in config.rule_applications(rule_id) {
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
            // Legacy statement policies select only SQL files until executors
            // are explicitly configured; do not request an empty TS projection.
            if *rule_id != "postgres-sql-statement-policy" || profile.selects_executors() {
                if *rule_id == "postgres-require-query-annotation" {
                    annotation_profiles.push(profile.clone());
                }
                let paths = scans::record(&mut grouped, profile, rule_id, &options)?;
                if *rule_id == "postgres-require-query-annotation" {
                    visit(rule, paths);
                }
            }
        }
    }
    annotation_profiles.sort();
    annotation_profiles.dedup();
    let profiles = grouped.keys().cloned().collect();
    Ok((profiles, annotation_profiles, grouped))
}
