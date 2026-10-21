use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::config::v2::NoMistakesConfig;

mod resolve;
mod route_index;
mod route_targets;
mod routes;
pub(crate) use resolve::{unique_export, unique_namespace_member, UniqueExport};
pub(crate) use route_index::RegistryRoutes;
pub(crate) use routes::registry_route;

pub(crate) fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    super::enabled(config).then_some(GraphBuildPlan {
        calls: true,
        imports: true,
        ..GraphBuildPlan::default()
    })
}

#[cfg(test)]
mod tests;
