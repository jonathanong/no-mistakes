use super::{flow_sequence_lines_from, is_content, value_after_anchor, FlowState};
use crate::codebase::rules::github_actions_step_timeouts::yaml::yaml_key_value;
use std::collections::BTreeMap;

fn mapping_key_len(text: &str, wanted: &str) -> Option<usize> {
    if text
        .strip_prefix(wanted)
        .is_some_and(|rest| rest.starts_with(':'))
    {
        return Some(wanted.len() + 1);
    }
    for quote in ['"', '\''] {
        if text.starts_with(quote)
            && text[quote.len_utf8()..]
                .strip_prefix(wanted)
                .is_some_and(|rest| {
                    rest.starts_with(quote) && rest[quote.len_utf8()..].starts_with(':')
                })
        {
            return Some(wanted.len() + 3);
        }
    }
    None
}

fn key_boundary(previous: Option<char>) -> bool {
    previous.is_none_or(|char| char.is_whitespace() || char == '{' || char == ',')
}

#[derive(Clone, Copy)]
struct JobMarker {
    line: usize,
    value_column: usize,
    alias: bool,
}

pub(crate) struct RootFlowIndex {
    jobs: BTreeMap<String, JobMarker>,
    jobs_alias_line: Option<usize>,
}

impl RootFlowIndex {
    pub(crate) fn new(lines: &[&str]) -> Self {
        let mut state = FlowState::default();
        let mut ignored = Vec::new();
        let mut jobs = BTreeMap::new();
        let mut found_jobs = false;
        let mut inside_jobs = false;
        let mut jobs_alias_line = None;
        'scan: for (line_index, line) in lines.iter().enumerate() {
            let mut previous = None;
            for (column, character) in line.char_indices() {
                if state.quote.is_none()
                    && character == '#'
                    && previous.is_none_or(char::is_whitespace)
                {
                    break;
                }
                if state.quote.is_none() && state.sequence_depth == 0 && key_boundary(previous) {
                    if !found_jobs
                        && state.mapping_depth == 1
                        && mapping_key_len(&line[column..], "jobs").is_some()
                    {
                        found_jobs = true;
                        let length = mapping_key_len(&line[column..], "jobs").unwrap_or(0);
                        let value = value_after_anchor(line[column + length..].trim_start());
                        if value.starts_with('*') {
                            jobs_alias_line = Some(line_index + 1);
                        }
                    } else if found_jobs && state.mapping_depth == 2 {
                        if let Some((job_id, _)) = yaml_key_value(&line[column..]) {
                            if let Some(colon) = line[column..].find(':') {
                                let value_column = column + colon + 1;
                                let alias = value_after_anchor(line[value_column..].trim_start())
                                    .starts_with('*');
                                jobs.entry(job_id.to_string()).or_insert(JobMarker {
                                    line: line_index,
                                    value_column,
                                    alias,
                                });
                            }
                        }
                    }
                }
                state.visit(character, line_index + 1, &mut ignored);
                inside_jobs |= found_jobs && state.mapping_depth == 2;
                if inside_jobs && state.mapping_depth == 1 {
                    break 'scan;
                }
                previous = Some(character);
            }
        }
        Self {
            jobs,
            jobs_alias_line,
        }
    }

    pub(crate) fn job_steps(&self, lines: &[&str], job_id: &str, count: usize) -> Vec<usize> {
        if let Some(line) = self.jobs_alias_line {
            return vec![line; count];
        }
        let Some(marker) = self.jobs.get(job_id) else {
            return vec![1; count];
        };
        if marker.alias || next_line_alias(lines, marker.line, marker.value_column) {
            return vec![marker.line + 1; count];
        }
        locate_job_steps(lines, marker.line, marker.value_column, count)
    }
}

fn next_line_alias(lines: &[&str], line: usize, value_column: usize) -> bool {
    if !lines[line][value_column..].trim().is_empty() {
        return false;
    }
    lines
        .iter()
        .skip(line + 1)
        .find(|line| is_content(line))
        .is_some_and(|line| value_after_anchor(line.trim_start()).starts_with('*'))
}

pub(crate) fn locate_job_steps(
    lines: &[&str],
    job_line: usize,
    value_column: usize,
    count: usize,
) -> Vec<usize> {
    let mut state = FlowState {
        mapping_depth: 2,
        ..FlowState::default()
    };
    let mut ignored = Vec::new();
    let mut inside_job = false;
    for (line_index, line) in lines.iter().enumerate().skip(job_line) {
        let text = if line_index == job_line {
            &line[value_column..]
        } else {
            line
        };
        let mut previous = None;
        for (offset, character) in text.char_indices() {
            if state.quote.is_none() && character == '#' && previous.is_none_or(char::is_whitespace)
            {
                break;
            }
            if state.quote.is_none()
                && state.sequence_depth == 0
                && state.mapping_depth == 3
                && key_boundary(previous)
            {
                if let Some(length) = mapping_key_len(&text[offset..], "steps") {
                    let value = &text[offset + length..];
                    let start = if value_after_anchor(value.trim_start()).starts_with('[') {
                        value.find('[').map(|column| {
                            (
                                line_index,
                                line.len() - text.len() + offset + length + column,
                            )
                        })
                    } else {
                        lines
                            .iter()
                            .enumerate()
                            .skip(line_index + 1)
                            .find(|(_, line)| is_content(line))
                            .and_then(|(index, line)| {
                                value_after_anchor(line.trim_start())
                                    .starts_with('[')
                                    .then(|| (index, line.find('[').unwrap_or(0)))
                            })
                    };
                    return start.map_or_else(
                        || vec![line_index + 1; count],
                        |(start, column)| {
                            flow_sequence_lines_from(lines, start, column, line_index + 1, count)
                        },
                    );
                }
            }
            state.visit(character, line_index + 1, &mut ignored);
            inside_job |= state.mapping_depth == 3;
            if inside_job && state.mapping_depth == 2 {
                return vec![job_line + 1; count];
            }
            previous = Some(character);
        }
    }
    vec![job_line + 1; count]
}
