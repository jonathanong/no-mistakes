//! A hand-written scanner for `${{ ... }}` GitHub Actions expressions,
//! ported from `expression-references.mts`. This is deliberately NOT a real
//! expression parser — it conservatively extracts `needs.<job>` /
//! `steps.<step>` accesses (dot or bracket form) and full
//! `needs.<job>.outputs.<name>` chains from `if:` conditions, tolerating
//! whitespace and quoted-string noise, and gives up (extracts nothing)
//! rather than guess on anything it doesn't recognize.
//!
//! Operates on `Vec<char>` rather than raw byte offsets: GitHub Actions
//! identifiers are ASCII, but a quoted string literal *within* an
//! expression can contain arbitrary UTF-8, and char-indexed scanning avoids
//! any risk of slicing on a non-ASCII byte boundary.

use char_scan::{
    embedded_expressions, has_further_access, is_access_boundary, previous_non_whitespace,
    quoted_end, static_access,
};
use serde_yaml::Value;
use std::collections::HashMap;

mod char_scan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticWorkflowOutputReference {
    pub call_job_id: String,
    pub output: String,
}

/// Extract every `needs.<x>` / `steps.<x>` access (dot or bracket form)
/// directly in `condition`, sorted and deduplicated. `context` is `"needs"`
/// or `"steps"` and is matched case-sensitively (unlike the output-chain
/// scanner below, which matches `needs` case-insensitively).
pub fn static_references(condition: Option<&str>, context: &str) -> Vec<String> {
    let Some(condition) = condition else {
        return Vec::new();
    };
    let chars: Vec<char> = condition.chars().collect();
    let context_chars: Vec<char> = context.chars().collect();
    let mut references = std::collections::BTreeSet::new();
    let mut index = 0usize;
    while index < chars.len() {
        let character = chars[index];
        if character == '\'' || character == '"' {
            index = quoted_end(&chars, index, character);
            continue;
        }
        let starts_with_context = chars[index..].starts_with(context_chars.as_slice());
        if !starts_with_context
            || !is_access_boundary(previous_non_whitespace(&chars, index as isize - 1))
        {
            index += 1;
            continue;
        }
        let access_start = index + context_chars.len();
        match static_access(&chars, access_start) {
            Some((reference, end)) => {
                references.insert(reference);
                index = end;
            }
            None => index = access_start,
        }
    }
    references.into_iter().collect()
}

/// Extract every `needs.<job>.outputs.<name>` chain. When `allow_bare` is
/// false (the default use), only chains inside `${{ ... }}` delimiters
/// count; when true, the whole string is scanned as one expression body —
/// used for `if:` conditions, which GitHub Actions allows to omit the
/// `${{ }}` wrapper entirely.
pub fn static_workflow_output_references(
    value: Option<&str>,
    allow_bare: bool,
) -> Vec<StaticWorkflowOutputReference> {
    let Some(value) = value else {
        return Vec::new();
    };
    let chars: Vec<char> = value.chars().collect();
    let expressions: Vec<Vec<char>> = if allow_bare {
        vec![chars]
    } else {
        embedded_expressions(&chars)
    };
    let mut references: HashMap<String, StaticWorkflowOutputReference> = HashMap::new();
    for expression in &expressions {
        collect_output_references(expression, &mut references);
    }
    sorted_references(references)
}

/// Recursively scan every string scalar reachable from `job` (a YAML
/// mapping) for `${{ needs.<job>.outputs.<name> }}` chains, plus a bare
/// (non-`${{ }}`-wrapped) scan of the job's own `if:` and each step's
/// `if:`. Matches the TS engine's `workflowOutputReferences`.
///
/// The TS original guards this walk with a `WeakSet` to avoid revisiting a
/// YAML-aliased subtree twice. `serde_yaml::Value` has no shared-identity
/// aliasing — an anchor/alias is fully materialized as an independent,
/// finite, acyclic copy at parse time — so that guard has no correctness
/// equivalent here: the recursion below is already finite by construction,
/// and any duplicated content is deduplicated the same way the TS engine
/// dedupes it (by composite `callJobId|output` key), not by object identity.
pub fn workflow_output_references(job: &Value) -> Vec<StaticWorkflowOutputReference> {
    let mut references: HashMap<String, StaticWorkflowOutputReference> = HashMap::new();
    visit_for_output_references(job, &mut references);

    if let Some(condition) = job.get("if").and_then(Value::as_str) {
        merge_references(
            &mut references,
            static_workflow_output_references(Some(condition), true),
        );
    }
    if let Some(steps) = job.get("steps").and_then(Value::as_sequence) {
        for step in steps {
            if let Some(condition) = step.get("if").and_then(Value::as_str) {
                merge_references(
                    &mut references,
                    static_workflow_output_references(Some(condition), true),
                );
            }
        }
    }

    sorted_references(references)
}

fn visit_for_output_references(
    value: &Value,
    references: &mut HashMap<String, StaticWorkflowOutputReference>,
) {
    match value {
        Value::String(text) => {
            merge_references(
                references,
                static_workflow_output_references(Some(text), false),
            );
        }
        Value::Sequence(items) => {
            for item in items {
                visit_for_output_references(item, references);
            }
        }
        Value::Mapping(mapping) => {
            for (_key, item) in mapping {
                visit_for_output_references(item, references);
            }
        }
        _ => {}
    }
}

fn merge_references(
    references: &mut HashMap<String, StaticWorkflowOutputReference>,
    found: Vec<StaticWorkflowOutputReference>,
) {
    for reference in found {
        references.entry(sort_key(&reference)).or_insert(reference);
    }
}

fn sorted_references(
    references: HashMap<String, StaticWorkflowOutputReference>,
) -> Vec<StaticWorkflowOutputReference> {
    let mut result: Vec<StaticWorkflowOutputReference> = references.into_values().collect();
    result.sort_by_key(sort_key);
    result
}

fn sort_key(reference: &StaticWorkflowOutputReference) -> String {
    format!(
        "{}|{}",
        reference.call_job_id.to_lowercase(),
        reference.output.to_lowercase()
    )
}

/// Scans one expression body for `needs.<job>.outputs.<name>` chains
/// (`needs` matched case-insensitively, unlike [`static_references`]).
fn collect_output_references(
    expression: &[char],
    references: &mut HashMap<String, StaticWorkflowOutputReference>,
) {
    let mut index = 0usize;
    while index < expression.len() {
        let character = expression[index];
        if character == '\'' || character == '"' {
            index = quoted_end(expression, index, character);
            continue;
        }
        let end = (index + 5).min(expression.len());
        let matches_needs = end - index == 5
            && expression[index..end]
                .iter()
                .collect::<String>()
                .eq_ignore_ascii_case("needs");
        if !matches_needs
            || !is_access_boundary(previous_non_whitespace(expression, index as isize - 1))
        {
            index += 1;
            continue;
        }
        let job = static_access(expression, index + 5);
        let outputs = job
            .as_ref()
            .and_then(|job| static_access(expression, job.1));
        let output = match &outputs {
            Some(outputs) if outputs.0.eq_ignore_ascii_case("outputs") => {
                static_access(expression, outputs.1)
            }
            _ => None,
        };
        let (Some(job), Some(_), Some(output)) = (&job, &outputs, &output) else {
            index += 5;
            continue;
        };
        if has_further_access(expression, output.1) {
            index += 5;
            continue;
        }
        let reference = StaticWorkflowOutputReference {
            call_job_id: job.0.clone(),
            output: output.0.clone(),
        };
        references.entry(sort_key(&reference)).or_insert(reference);
        index = output.1;
    }
}
