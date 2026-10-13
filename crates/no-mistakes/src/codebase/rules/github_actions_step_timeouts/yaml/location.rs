use super::{is_inline_value_line, value_after_anchor, yaml_key_value};
use std::collections::BTreeMap;

mod parallel;
use parallel::parallel_child_lines;
mod block;
use block::{find_jobs_line, find_steps_line, index_job_lines};
mod flow;
use flow::{flow_sequence_item_lines, locate_job_steps, RootFlowIndex};

fn indent(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

fn is_content(line: &str) -> bool {
    let line = line.split('#').next().unwrap_or("").trim();
    !line.is_empty() && !(line.starts_with('&') && line.split_whitespace().count() == 1)
}

fn is_document_marker(line: &str) -> bool {
    let text = line.trim_start().trim_start_matches('\u{feff}');
    let Some(rest) = text.strip_prefix("---") else {
        return false;
    };
    rest.is_empty()
        || (rest.starts_with(char::is_whitespace)
            && (rest.trim_start().is_empty() || rest.trim_start().starts_with('#')))
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
    job_lines: BTreeMap<String, (usize, usize)>,
    root_flow: Option<RootFlowIndex>,
}

impl<'a> StepLocations<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        let lines: Vec<_> = source.lines().collect();
        let jobs = find_jobs_line(&lines);
        let root_flow = if flow::is_root_flow(&lines) {
            Some(RootFlowIndex::new(&lines))
        } else {
            jobs.and_then(|jobs| flow::jobs_flow_start(&lines, jobs))
                .map(|(line, column)| RootFlowIndex::jobs_flow(&lines, line, column))
        };
        let job_lines = jobs
            .map(|jobs| index_job_lines(&lines, jobs))
            .unwrap_or_default();
        Self {
            lines,
            jobs,
            job_lines,
            root_flow,
        }
    }

    pub(crate) fn job_steps(&self, job_id: &str, count: usize) -> Vec<usize> {
        if let Some(root_flow) = &self.root_flow {
            return root_flow.job_steps(&self.lines, job_id, count);
        }
        let Some(jobs) = self.jobs else {
            return vec![1; count];
        };
        if is_inline_value_line(self.lines[jobs.0])
            || has_child_inline_value(&self.lines, jobs, '{')
        {
            return vec![jobs.0 + 1; count];
        }
        let Some(job) = self.job_lines.get(job_id).copied() else {
            return vec![1; count];
        };
        if let Some((_, value)) = yaml_key_value(self.lines[job.0]) {
            if value_after_anchor(value).starts_with('{') {
                if let Some(column) = self.lines[job.0].find('{') {
                    return locate_job_steps(&self.lines, job.0, column, count);
                }
            }
        }
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
        let item = self.lines[parent].trim_start();
        let item = item.strip_prefix("- ").unwrap_or(item);
        if value_after_anchor(item).starts_with('{')
            || is_inline_value_line(self.lines[parent])
            || has_child_inline_value(&self.lines, key, '[')
            || has_child_inline_value(&self.lines, key, '{')
        {
            return flow_sequence_item_lines(&self.lines, key, count);
        }
        parallel_child_lines(&self.lines, parent_line, count)
    }
}
