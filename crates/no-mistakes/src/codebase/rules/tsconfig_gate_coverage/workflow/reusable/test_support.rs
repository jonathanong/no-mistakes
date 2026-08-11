use super::*;

pub(super) fn collect_ci_projects_with_stats(
    parsed: &ParsedWorkflowSet,
    tracked: &BTreeSet<String>,
    project_source_inputs: &ProjectSourceInputs,
) -> (BTreeSet<String>, usize) {
    collect_ci_projects_with_local_actions(parsed, tracked, project_source_inputs, &BTreeSet::new())
}
