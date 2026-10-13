use super::{indent, is_content};
use crate::codebase::rules::github_actions_step_timeouts::yaml::yaml_key_value;
use std::collections::BTreeMap;

pub(super) fn find_jobs_line(lines: &[&str]) -> Option<(usize, usize)> {
    let root_indent = lines
        .iter()
        .find(|line| {
            let text = line.trim_start();
            is_content(line) && text != "---" && !text.starts_with('%')
        })
        .map(|line| indent(line))?;
    lines.iter().enumerate().find_map(|(index, line)| {
        (indent(line) == root_indent && yaml_key_value(line).is_some_and(|(key, _)| key == "jobs"))
            .then_some((index, root_indent))
    })
}

pub(super) fn index_job_lines(
    lines: &[&str],
    jobs: (usize, usize),
) -> BTreeMap<String, (usize, usize)> {
    let mut child_indent = None;
    let mut explicit_key = None;
    let mut result = BTreeMap::new();
    for (index, line) in lines.iter().enumerate().skip(jobs.0 + 1) {
        if !is_content(line) {
            continue;
        }
        let depth = indent(line);
        if depth <= jobs.1 {
            break;
        }
        let direct_depth = *child_indent.get_or_insert(depth);
        if depth == direct_depth {
            let text = line.trim_start();
            if let Some(key) = text.strip_prefix("? ") {
                explicit_key = Some(key.trim().trim_matches(['\'', '"']).to_string());
            } else if text == ":" || text.starts_with(": ") {
                if let Some(key) = explicit_key.take() {
                    result.insert(key, (index, depth));
                }
            } else if let Some((key, _)) = yaml_key_value(line) {
                result.insert(key.to_string(), (index, depth));
            }
        }
    }
    result
}

pub(super) fn find_steps_line(lines: &[&str], job: (usize, usize)) -> Option<(usize, usize)> {
    let mut child_indent = None;
    for (index, line) in lines.iter().enumerate().skip(job.0 + 1) {
        if !is_content(line) {
            continue;
        }
        let depth = indent(line);
        if depth <= job.1 {
            break;
        }
        let direct_depth = *child_indent.get_or_insert(depth);
        if depth == direct_depth && yaml_key_value(line).is_some_and(|(key, _)| key == "steps") {
            return Some((index, depth));
        }
    }
    None
}
