pub(super) fn input_line(source: &str, job_id: &str, index: usize, key: &str) -> usize {
    super::super::github_actions_workflow_files::step_lines::first_key_line(
        source,
        job_id,
        index,
        &[key, "uses"],
    )
}
