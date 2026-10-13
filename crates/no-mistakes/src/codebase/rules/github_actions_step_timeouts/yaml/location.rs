use super::{is_inline_value_line, value_after_anchor, yaml_key_value};

mod parallel;
use parallel::parallel_child_lines;
mod flow;
use flow::flow_sequence_item_lines;

fn indent(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

fn is_content(line: &str) -> bool {
    let line = line.split('#').next().unwrap_or("").trim();
    !line.is_empty() && !(line.starts_with('&') && line.split_whitespace().count() == 1)
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

pub(super) fn sequence_item_lines(lines: &[&str], key: (usize, usize), count: usize) -> Vec<usize> {
    let mut item_indent = None;
    let mut result = Vec::with_capacity(count);
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
            result.push(index + 1);
            if result.len() == count {
                break;
            }
        }
    }
    result.resize(count, 1);
    result
}

// Return an inline declaration for aliases and flow collections; otherwise
// locate the authored sequence item so line suppressions remain precise.
pub(crate) struct StepLocations<'a> {
    lines: Vec<&'a str>,
    jobs: Option<(usize, usize)>,
}

impl<'a> StepLocations<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        let lines: Vec<_> = source.lines().collect();
        let jobs = find_jobs_line(&lines);
        Self { lines, jobs }
    }

    pub(crate) fn job_steps(&self, job_id: &str, count: usize) -> Vec<usize> {
        let Some(jobs) = self.jobs else {
            return vec![1; count];
        };
        if is_inline_value_line(self.lines[jobs.0])
            || has_child_inline_value(&self.lines, jobs, '{')
        {
            return vec![jobs.0 + 1; count];
        }
        let Some(job) = find_job_line(&self.lines, jobs, job_id) else {
            return vec![1; count];
        };
        if is_inline_value_line(self.lines[job.0]) || has_child_inline_value(&self.lines, job, '{')
        {
            return vec![job.0 + 1; count];
        }
        let Some(steps) = find_steps_line(&self.lines, job) else {
            return vec![1; count];
        };
        if is_inline_value_line(self.lines[steps.0])
            || has_child_inline_value(&self.lines, steps, '[')
        {
            return flow_sequence_item_lines(&self.lines, steps, count);
        }
        sequence_item_lines(&self.lines, steps, count)
    }

    pub(crate) fn parallel_steps(&self, parent_line: usize, count: usize) -> Vec<usize> {
        let Some(parent) = parent_line
            .checked_sub(1)
            .filter(|index| *index < self.lines.len())
        else {
            return vec![1; count];
        };
        let key = (parent, indent(self.lines[parent]));
        if is_inline_value_line(self.lines[parent])
            || has_child_inline_value(&self.lines, key, '[')
            || has_child_inline_value(&self.lines, key, '{')
        {
            return flow_sequence_item_lines(&self.lines, key, count);
        }
        parallel_child_lines(&self.lines, parent_line, count)
    }
}
