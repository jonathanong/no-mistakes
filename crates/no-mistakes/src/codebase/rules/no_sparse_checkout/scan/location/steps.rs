use super::{is_checkout_reference, is_sequence_entry, leading_indent, step_line, yaml_key_value};
use serde_yaml::{Mapping, Value};

pub(super) struct CheckoutStep {
    start: usize,
    end: usize,
    flow_start: Option<usize>,
    mapping_indent: usize,
}

impl CheckoutStep {
    pub(super) fn key_line(&self, lines: &[&str], code_lines: &[&str], key: &str) -> usize {
        if let Some(flow_start) = self.flow_start {
            return flow_start + 1;
        }
        let Some(with_line) = direct_key_line(
            lines,
            code_lines,
            self.start,
            self.end,
            self.mapping_indent,
            "with",
        ) else {
            return self.checkout_line(lines, code_lines) + 1;
        };
        let Some(input_indent) = child_mapping_indent(lines, code_lines, with_line, self.end)
        else {
            return self.checkout_line(lines, code_lines) + 1;
        };
        (with_line + 1..self.end)
            .find(|index| {
                leading_indent(lines[*index]) == input_indent
                    && yaml_key_value(code_lines[*index].trim(), key).is_some()
            })
            .map_or_else(
                || self.checkout_line(lines, code_lines) + 1,
                |index| index + 1,
            )
    }

    fn checkout_line(&self, lines: &[&str], code_lines: &[&str]) -> usize {
        direct_key_line(
            lines,
            code_lines,
            self.start,
            self.end,
            self.mapping_indent,
            "uses",
        )
        .unwrap_or(self.start)
    }
}

pub(super) fn checkout_steps<'a>(
    lines: &'a [&'a str],
    code_lines: &'a [&'a str],
) -> impl Iterator<Item = CheckoutStep> + 'a {
    (0..lines.len()).filter_map(|start| {
        let end = step_end(lines, code_lines, start)?;
        let mapping = parse_step_mapping(lines, start, end)?;
        mapping
            .get("uses")
            .and_then(Value::as_str)
            .is_some_and(is_checkout_reference)
            .then(|| {
                let flow_start = (start..end)
                    .find(|index| step_line(code_lines[*index], *index == start).starts_with('{'));
                let mapping_indent = flow_start
                    .map(|index| leading_indent(lines[index]))
                    .unwrap_or_else(|| step_mapping_indent(lines, code_lines, start, end));
                CheckoutStep {
                    start,
                    end,
                    flow_start,
                    mapping_indent,
                }
            })
    })
}

fn step_end(lines: &[&str], code_lines: &[&str], start: usize) -> Option<usize> {
    let line = code_lines.get(start)?.trim_start();
    if !is_sequence_entry(line) || !belongs_to_steps(lines, code_lines, start) {
        return None;
    }
    let step_indent = leading_indent(lines[start]);
    Some(
        (start + 1..lines.len())
            .find(|index| {
                let line = code_lines[*index].trim();
                !line.is_empty()
                    && (leading_indent(lines[*index]) < step_indent
                        || (leading_indent(lines[*index]) <= step_indent
                            && is_sequence_entry(line)))
            })
            .unwrap_or(lines.len()),
    )
}

fn belongs_to_steps(lines: &[&str], code_lines: &[&str], start: usize) -> bool {
    let step_indent = leading_indent(lines[start]);
    (0..start)
        .rev()
        .find(|index| {
            let line = code_lines[*index].trim();
            !line.is_empty()
                && leading_indent(lines[*index]) <= step_indent
                && (leading_indent(lines[*index]) < step_indent || !is_sequence_entry(line))
        })
        .is_some_and(|index| yaml_key_value(code_lines[index].trim(), "steps").is_some())
}

fn parse_step_mapping(lines: &[&str], start: usize, end: usize) -> Option<Mapping> {
    let indent = leading_indent(lines[start]);
    let document = lines[start..end]
        .iter()
        .map(|line| line.get(indent..).unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    serde_yaml::from_str::<Vec<Value>>(&document)
        .ok()?
        .into_iter()
        .next()?
        .as_mapping()
        .cloned()
}

fn step_mapping_indent(lines: &[&str], code_lines: &[&str], start: usize, end: usize) -> usize {
    if !step_line(code_lines[start], true).is_empty() {
        return leading_indent(lines[start]) + 2;
    }
    (start + 1..end)
        .find(|index| !code_lines[*index].trim().is_empty())
        .map_or(leading_indent(lines[start]) + 2, |index| {
            leading_indent(lines[index])
        })
}

fn direct_key_line(
    lines: &[&str],
    code_lines: &[&str],
    start: usize,
    end: usize,
    mapping_indent: usize,
    key: &str,
) -> Option<usize> {
    (start..end).find(|index| {
        let line = step_line(code_lines[*index], *index == start);
        let indent = if *index == start {
            leading_indent(lines[*index]) + 2
        } else {
            leading_indent(lines[*index])
        };
        indent == mapping_indent && yaml_key_value(line, key).is_some()
    })
}

fn child_mapping_indent(
    lines: &[&str],
    code_lines: &[&str],
    parent: usize,
    end: usize,
) -> Option<usize> {
    (parent + 1..end)
        .find(|index| !code_lines[*index].trim().is_empty())
        .map(|index| leading_indent(lines[index]))
        .filter(|indent| *indent > leading_indent(lines[parent]))
}
