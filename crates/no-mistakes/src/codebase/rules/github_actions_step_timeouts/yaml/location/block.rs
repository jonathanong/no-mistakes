use super::{indent, is_content};
use crate::codebase::rules::github_actions_step_timeouts::yaml::yaml_key_value;
use std::collections::BTreeMap;

fn explicit_key_name(line: &str) -> Option<&str> {
    line.trim_start()
        .strip_prefix("? ")
        .map(str::trim)
        .map(|key| key.trim_matches(['\'', '"']))
}

fn explicit_value(line: &str) -> bool {
    let text = line.trim_start();
    text == ":" || text.starts_with(": ")
}

pub(super) fn find_jobs_line(lines: &[&str]) -> Option<(usize, usize)> {
    let root_indent = lines
        .iter()
        .find(|line| {
            let text = line.trim_start();
            is_content(line) && text != "---" && !text.starts_with('%')
        })
        .map(|line| indent(line))?;
    let mut explicit_jobs = false;
    for (index, line) in lines.iter().enumerate() {
        if indent(line) != root_indent {
            continue;
        }
        if yaml_key_value(line).is_some_and(|(key, _)| key == "jobs")
            || (explicit_jobs && explicit_value(line))
        {
            return Some((index, root_indent));
        }
        explicit_jobs = explicit_key_name(line) == Some("jobs");
    }
    None
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
            if let Some(key) = explicit_key_name(line) {
                explicit_key = Some(key.to_string());
            } else if explicit_value(line) {
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
    let mut explicit_steps = false;
    for (index, line) in lines.iter().enumerate().skip(job.0 + 1) {
        if !is_content(line) {
            continue;
        }
        let depth = indent(line);
        if depth <= job.1 {
            break;
        }
        let direct_depth = *child_indent.get_or_insert(depth);
        if depth == direct_depth {
            if yaml_key_value(line).is_some_and(|(key, _)| key == "steps")
                || (explicit_steps && explicit_value(line))
            {
                return Some((index, depth));
            }
            explicit_steps = explicit_key_name(line) == Some("steps");
        }
    }
    None
}
