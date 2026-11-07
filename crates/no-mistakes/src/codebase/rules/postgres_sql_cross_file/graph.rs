use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::config::v2::NoMistakesConfig;

mod resolve;
mod route_index;
mod route_targets;
mod routes;
pub(crate) use resolve::{
    unique_export, unique_namespace_member, unique_type_export, UniqueExport,
};
pub(crate) use route_index::RegistryRoutes;
pub(crate) use routes::registry_route;

pub(crate) fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    super::needs_deep_analysis(config).then_some(GraphBuildPlan {
        calls: true,
        imports: true,
        effect_selectors: needs_module_validators(config),
        ..GraphBuildPlan::default()
    })
}

fn needs_module_validators(config: &NoMistakesConfig) -> bool {
    config
        .rule_applications("postgres-no-unbounded-sql-identifier")
        .iter()
        .any(|rule| {
            config.effective_rule_analysis(rule).map_or(true, |policy| {
                policy.effort == crate::config::v2::Effort::Max
            }) && rule
                .options
                .get("identifierValidators")
                .and_then(serde_yaml::Value::as_sequence)
                .is_some_and(|validators| {
                    validators.iter().any(|validator| {
                        validator
                            .get("module")
                            .and_then(serde_yaml::Value::as_str)
                            .is_some_and(|module| !module.is_empty())
                    })
                })
        })
}

#[cfg(test)]
mod tests;
