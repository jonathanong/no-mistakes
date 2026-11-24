use super::yaml::{finding, literal_u64, mapping_get, nested_message, yaml_got};
use super::{CompiledOptions, RuleFinding};
use serde_yaml::{Mapping, Value};

pub(super) fn nested_input_finding(
    rel: &str,
    source: &str,
    job_id: &str,
    label: &str,
    raw: Option<&Value>,
    index: usize,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    if opts.nested_input.is_empty() {
        return Vec::new();
    }
    let Some(expected) = opts.nested_timeout_seconds else {
        return Vec::new();
    };
    if raw.and_then(literal_u64) == Some(expected) {
        return Vec::new();
    }
    vec![finding(
        rel,
        nested_timeout_line(source, job_id, index, &opts.nested_input),
        nested_message(
            rel,
            job_id,
            label,
            &opts.nested_input,
            expected,
            &yaml_got(raw),
        ),
        job_id,
    )]
}

pub(super) fn nested_input_value<'a>(step: &'a Mapping, nested_input: &str) -> Option<&'a Value> {
    mapping_get(step, "with")
        .and_then(Value::as_mapping)
        .and_then(|with| mapping_get(with, nested_input))
}

pub(super) fn nested_timeout_line(
    source: &str,
    job_id: &str,
    index: usize,
    nested_input: &str,
) -> usize {
    super::super::github_actions_workflow_files::step_lines::first_key_line(
        source,
        job_id,
        index,
        &[nested_input, "uses", "with"],
    )
}

pub(super) fn step_key_line(source: &str, job_id: &str, index: usize, key: &str) -> usize {
    super::super::github_actions_workflow_files::step_lines::first_key_line(
        source,
        job_id,
        index,
        &[key, "uses"],
    )
}
