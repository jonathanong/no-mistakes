use super::{RuleFinding, RULE_ID};
use regex::Regex;
use serde_yaml::Value;
use std::collections::BTreeSet;
use std::sync::Arc;

mod carry;
mod indent;
mod lines;

pub(super) fn is_block_header(scalar: &str) -> bool {
    carry::is_block_header(scalar)
}

/// Pair job and step `uses` values, in document order, with action `uses`
/// lines. The regex already judged `seen_lines`. Once a document parses, a
/// `uses` line under `with` or `env` is an input and is dropped. A missing
/// document leaves the line-scan findings untouched.
pub(super) fn append_parsed_uses(
    content: &str,
    documents: &[Arc<Value>],
    file: &str,
    sha_re: &Regex,
    version_re: &Regex,
    seen_lines: &BTreeSet<usize>,
    findings: &mut Vec<RuleFinding>,
) {
    if documents.is_empty() {
        return;
    }
    let mut uses_values = Vec::new();
    for document in documents {
        collect_uses(document, &mut uses_values);
    }
    let source_lines: Vec<&str> = content.lines().collect();
    let classified = lines::action_uses_lines(&source_lines);
    // A `uses` inside `with` or `env` is an input. Block indent and a flow
    // mapping that continues on a later line are both inputs.
    findings.retain(|finding| {
        let index = finding.line.saturating_sub(1);
        let flow_input = classified
            .flow_input_at_start
            .get(index)
            .copied()
            .unwrap_or(false);
        let in_block = classified.block_scalar.get(index).copied().unwrap_or(false);
        !indent::in_input_block(&source_lines, index) && !flow_input && !in_block
    });
    let action_lines = classified.lines;
    for (uses_value, line_num) in uses_values.iter().zip(action_lines) {
        if seen_lines.contains(&line_num) {
            continue;
        }
        if exempt(uses_value) {
            continue;
        }
        let comment = version_comment(source_lines[line_num - 1]);
        if let Some(finding) =
            unpinned_finding(file, line_num, uses_value, sha_re, version_re, comment)
        {
            findings.push(finding);
        }
    }
}

pub(super) fn exempt(uses_value: &str) -> bool {
    uses_value.starts_with("./")
        || uses_value.starts_with("../")
        || uses_value.starts_with("docker://")
}

pub(super) fn unpinned_finding(
    file: &str,
    line_num: usize,
    uses_value: &str,
    sha_re: &Regex,
    version_re: &Regex,
    comment: &str,
) -> Option<RuleFinding> {
    let ref_part = uses_value.rsplit_once('@').map_or("", |(_, part)| part);
    let sha_ok = sha_re.is_match(ref_part);
    let comment_ok = version_re.is_match(comment);
    if sha_ok && comment_ok {
        return None;
    }
    let reason = if !sha_ok {
        format!("`{ref_part}` is not a 40-char commit SHA")
    } else {
        format!("trailing comment must be `# v1.2.3` or `# 1.87.0`, got `{comment}`")
    };
    Some(RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: line_num,
        message: format!(
            "{file}:{line_num}: `uses: {uses_value}` — {reason} \
             (pin with `owner/repo@<40-char-sha> # v1.2.3`)"
        ),
        import: None,
        target: None,
    })
}

/// Version comment on the line that contains the value text, not a bare `uses:`.
fn version_comment(line: &str) -> &str {
    let mut quote = None;
    for (index, ch) in line.char_indices() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '#' => return line[index..].trim(),
            _ => {}
        }
    }
    ""
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Place {
    Root,
    Jobs,
    Job,
    Steps,
    Step,
}

fn collect_uses(value: &Value, out: &mut Vec<String>) {
    collect_in(value, Place::Root, true, out);
}

/// Job `uses` and step `uses` only. A root sequence is a step list (the saved
/// fixtures). Inputs under `with` or `env` are not action references.
fn collect_in(value: &Value, place: Place, at_root: bool, out: &mut Vec<String>) {
    match value {
        Value::Mapping(mapping) => {
            if matches!(place, Place::Job | Place::Step) {
                for (key, child) in mapping {
                    if key.as_str() == Some("uses") {
                        if let Some(text) = child.as_str() {
                            let text = text.trim();
                            if !text.is_empty() {
                                out.push(text.to_owned());
                            }
                        }
                    }
                }
            }
            for (key, child) in mapping {
                let Some(key) = key.as_str() else {
                    continue;
                };
                if matches!(key, "with" | "env") {
                    continue;
                }
                let next = match key {
                    "jobs" => Place::Jobs,
                    "steps" => Place::Steps,
                    _ if place == Place::Jobs => Place::Job,
                    _ => Place::Root,
                };
                collect_in(child, next, false, out);
            }
        }
        Value::Sequence(items) => {
            let as_steps = place == Place::Steps || (at_root && place == Place::Root);
            for item in items {
                if as_steps {
                    collect_in(item, Place::Step, false, out);
                } else {
                    collect_in(item, place, false, out);
                }
            }
        }
        _ => {}
    }
}
