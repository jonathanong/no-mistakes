use crate::codebase::rules::github_actions_workflow_files::job_lines::document::Document;

pub(super) fn references(script: &str, name: &str) -> bool {
    script.match_indices('$').any(|(index, _)| {
        let rest = &script[index + 1..];
        if let Some(braced) = rest.strip_prefix('{') {
            return braced
                .strip_prefix(name)
                .is_some_and(|end| end.starts_with('}'));
        }
        rest.strip_prefix(name).is_some_and(|end| {
            end.bytes()
                .next()
                .is_none_or(|byte| byte != b'_' && !byte.is_ascii_alphanumeric())
        })
    })
}

pub(super) fn line(
    document: &Document,
    job_name: &str,
    step_index: usize,
    name: &str,
) -> Option<usize> {
    let root = *document.roots().first()?;
    let job = document
        .field(root, "jobs")
        .and_then(|jobs| document.field(jobs, job_name));
    let step = job
        .and_then(|job| document.field(job, "steps"))
        .and_then(|steps| document.sequence(steps))
        .and_then(|steps| steps.get(step_index))
        .copied();
    [step, job, Some(root)]
        .into_iter()
        .flatten()
        .find_map(|owner| document.field_line(document.field(owner, "env")?, name))
}
