use super::{is_inline_value_line, value_after_anchor, yaml_key_value};

mod parallel;
use parallel::parallel_child_line;

fn indent(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

fn is_content(line: &str) -> bool {
    let line = line.trim_start();
    !line.is_empty() && !line.starts_with('#')
}

pub(super) fn has_child_inline_value(lines: &[&str], key: (usize, usize), opening: char) -> bool {
    lines
        .iter()
        .skip(key.0 + 1)
        .find(|line| is_content(line))
        .is_some_and(|line| {
            let value = value_after_anchor(line.trim_start());
            indent(line) > key.1 && (value.starts_with(opening) || value.starts_with('*'))
        })
}

fn find_jobs_line(lines: &[&str]) -> Option<(usize, usize)> {
    let root_indent = lines
        .iter()
        .find(|line| is_content(line) && line.trim() != "---")
        .map(|line| indent(line))?;
    lines.iter().enumerate().find_map(|(index, line)| {
        (indent(line) == root_indent && yaml_key_value(line).is_some_and(|(key, _)| key == "jobs"))
            .then_some((index, root_indent))
    })
}

fn find_job_line(lines: &[&str], jobs: (usize, usize), job_id: &str) -> Option<(usize, usize)> {
    let mut child_indent = None;
    for (index, line) in lines.iter().enumerate().skip(jobs.0 + 1) {
        if !is_content(line) {
            continue;
        }
        let depth = indent(line);
        if depth <= jobs.1 {
            break;
        }
        let direct_depth = *child_indent.get_or_insert(depth);
        if depth == direct_depth && yaml_key_value(line).is_some_and(|(key, _)| key == job_id) {
            return Some((index, depth));
        }
    }
    None
}

fn find_steps_line(lines: &[&str], job: (usize, usize)) -> Option<(usize, usize)> {
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

pub(super) fn sequence_item_line(
    lines: &[&str],
    key: (usize, usize),
    wanted: usize,
) -> Option<usize> {
    let mut item_indent = None;
    let mut seen = 0;
    for (index, line) in lines.iter().enumerate().skip(key.0 + 1) {
        if !is_content(line) {
            continue;
        }
        let depth = indent(line);
        let trimmed = line.trim_start();
        let is_item = trimmed.starts_with("- ") || trimmed == "-";
        if depth < key.1 || (depth == key.1 && !is_item) {
            break;
        }
        if !is_item {
            continue;
        }
        let item_depth = *item_indent.get_or_insert(depth);
        if depth == item_depth {
            if seen == wanted {
                return Some(index + 1);
            }
            seen += 1;
        }
    }
    None
}

// Return an inline declaration for aliases and flow collections; otherwise
// locate the authored sequence item so line suppressions remain precise.
pub(super) fn step_line(source: &str, job_id: &str, index: usize) -> Option<usize> {
    let lines: Vec<_> = source.lines().collect();
    let jobs = find_jobs_line(&lines)?;
    if is_inline_value_line(lines[jobs.0]) || has_child_inline_value(&lines, jobs, '{') {
        return Some(jobs.0 + 1);
    }
    let job = find_job_line(&lines, jobs, job_id)?;
    if is_inline_value_line(lines[job.0]) || has_child_inline_value(&lines, job, '{') {
        return Some(job.0 + 1);
    }
    let steps = find_steps_line(&lines, job)?;
    if is_inline_value_line(lines[steps.0]) || has_child_inline_value(&lines, steps, '[') {
        return Some(steps.0 + 1);
    }
    sequence_item_line(&lines, steps, index)
}

pub(crate) fn step_line_path(source: &str, job_id: &str, path: &[usize]) -> Option<usize> {
    let (first, rest) = path.split_first()?;
    let mut line = step_line(source, job_id, *first)?;
    let lines: Vec<_> = source.lines().collect();
    let key = (line - 1, indent(lines[line - 1]));
    if is_inline_value_line(lines[key.0])
        || has_child_inline_value(&lines, key, '[')
        || has_child_inline_value(&lines, key, '{')
    {
        return Some(line);
    }
    for index in rest {
        line = parallel_child_line(source, line, *index)?;
    }
    Some(line)
}
