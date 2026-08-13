use super::*;

pub(super) fn collect_ci_projects_with_stats(
    parsed: &ParsedWorkflowSet,
    tracked: &BTreeSet<String>,
    project_source_inputs: &ProjectSourceInputs,
) -> (BTreeSet<String>, usize) {
    let tracked_paths = tracked
        .iter()
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>();
    collect_ci_projects_with_local_actions(
        std::path::Path::new("."),
        parsed,
        tracked,
        &tracked_paths,
        project_source_inputs,
        &super::super::local_actions::LocalActionCatalog::default(),
    )
}
