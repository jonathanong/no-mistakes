use super::*;
use crate::codebase::dependencies::graph::GraphBuildPlan;

/// Run the `rsc-callers` query.
pub fn run(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig: Option<&Path>,
    component: &Path,
    depth: Option<usize>,
) -> Result<RscCallersReport> {
    let root = normalize_path(root);
    let root = root.canonicalize().unwrap_or(root);
    let component_abs = if component.is_absolute() {
        component.to_path_buf()
    } else {
        root.join(component)
    };
    if !component_abs.is_file() {
        anyhow::bail!("component file not found: {}", component_abs.display());
    }
    let crate::codebase::dependencies::query::PreparedQueryGraph {
        graph,
        facts,
        interner,
        ..
    } = crate::codebase::dependencies::query::prepare_query_graph(
        crate::codebase::dependencies::query::QueryGraphRequest {
            root: &root,
            config_path,
            tsconfig_path: tsconfig,
            explicit_root: &component_abs,
            config_before_facts: false,
            fact_plan: crate::codebase::ts_source::facts::TsFactPlan {
                imports: true,
                function_calls: true,
                rsc_environment: true,
                ..Default::default()
            },
            select: |_: Option<&crate::config::v2::schema::NoMistakesConfig>| anyhow::Ok(()),
            validate_root: |_: &(), _: &mut crate::codebase::dependencies::graph::GraphFiles| Ok(()),
            graph_plan: |_: &(),
                         _: &crate::codebase::dependencies::graph::GraphFiles,
                         _: &crate::codebase::ts_source::facts::TsFactMap| {
                GraphBuildPlan::from_allowed(Some(&runtime_edges()))
            },
            configure_context:
                |_: &(), _: &mut crate::codebase::ts_source::facts::TsFactContext| {},
            selection: std::marker::PhantomData,
        },
    )?;

    run_with_prepared(&root, component, depth, &graph, &facts, &interner)
}
