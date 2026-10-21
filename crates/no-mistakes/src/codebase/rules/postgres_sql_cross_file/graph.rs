use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::config::v2::NoMistakesConfig;

mod resolve;
pub(crate) use resolve::{unique_export, unique_namespace_member, UniqueExport};

pub(crate) fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    super::enabled(config).then_some(GraphBuildPlan {
        calls: true,
        imports: true,
        ..GraphBuildPlan::default()
    })
}

#[cfg(test)]
mod tests;
