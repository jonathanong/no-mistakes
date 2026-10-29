use super::{
    assign_prepared_embedded_sql_options, configure_prepared_postgres_plan, CheckFactPlan,
    NoMistakesConfig, RuleDef, RuleScope,
};

pub(super) fn config(efforts: &[&str]) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    for effort in efforts {
        config.rules.push(RuleDef {
            rule: "postgres-require-query-annotation".into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(&format!(
                "effort: {effort}\nimportSpecifier: '@bench/db'\nexecutorNames: [read]"
            ))
            .unwrap(),
            ..Default::default()
        });
    }
    crate::codebase::rules::analysis_policy::prepare_rule_analysis(
        std::path::Path::new("."),
        &mut config,
        &[],
        &crate::codebase::ts_source::SourceStore::new(std::sync::Arc::new(
            crate::codebase::ts_source::FileInventory::from_paths(&[]),
        )),
    )
    .unwrap();
    config
}

pub(super) fn plan(root: &std::path::Path, config: &NoMistakesConfig) -> CheckFactPlan {
    let mut plan = CheckFactPlan::default();
    configure_prepared_postgres_plan(root, config, &mut plan).unwrap();
    assign_prepared_embedded_sql_options(config, &mut plan).unwrap();
    plan
}
