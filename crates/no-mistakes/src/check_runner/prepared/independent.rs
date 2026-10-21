use std::path::{Path, PathBuf};
use std::sync::Arc;

type PreparedIndependent = (
    no_mistakes::codebase::config::InferredRoots,
    no_mistakes::codebase::config::Config,
    no_mistakes::react_traits::PreparedReactCheck,
    Option<Arc<no_mistakes::codebase::ci_workflows::PreparedWorkflowTopologyInputs>>,
    Option<no_mistakes::codebase::rules::tsconfig_gate_coverage::ProjectSourceInputs>,
);

pub(super) fn prepare(
    root: &Path,
    config_path: Option<&Path>,
    root_paths: &[PathBuf],
    config: &no_mistakes::config::v2::NoMistakesConfig,
    visible_paths: &no_mistakes::codebase::ts_source::VisiblePathSnapshot,
    sources: &no_mistakes::codebase::ts_source::SourceStore,
    workspace: Option<&no_mistakes::codebase::workspaces::IndexedWorkspaceMap>,
) -> PreparedIndependent {
    let inferred_roots =
        no_mistakes::codebase::config::InferredRoots::from_visible(root, root_paths);
    let codebase_config =
        no_mistakes::codebase::config::config_from_loaded_v2(root, config_path, config);
    let react = no_mistakes::react_traits::prepare_check_from_loaded_config(config, false);
    let workflow_documents =
        no_mistakes::codebase::ci_workflows::PreparedWorkflowTopologyInputs::for_config(
            root,
            config,
            visible_paths,
            sources,
        )
        .map(Arc::new);
    let tsconfig_gate_project_inputs = config
        .rule_configured(no_mistakes::codebase::rules::TSCONFIG_GATE_COVERAGE)
        .then(|| {
            no_mistakes::codebase::rules::tsconfig_gate_coverage::prepare_project_source_inputs(
                root,
                root_paths,
                sources,
                workspace.expect("tsconfig gate coverage requires a workspace projection"),
            )
        });
    (
        inferred_roots,
        codebase_config,
        react,
        workflow_documents,
        tsconfig_gate_project_inputs,
    )
}
