use super::values::{filter_predicates, parse_filters_value_with_sources};
use super::workflow_paths::WorkflowPathFilters;
use super::{workflow_finding, CiFilter, RuleFinding};
use crate::codebase::rules::vitest_ci_path_coverage::globs::{
    compile_pattern_predicates, PredicateQuantifier,
};
use serde_yaml::Value;
use std::path::Path;

pub(super) struct StepContext<'a> {
    pub(super) rel: &'a str,
    pub(super) job_id: &'a str,
    pub(super) step_id: &'a str,
    pub(super) workflow_paths: &'a WorkflowPathFilters,
}

pub(super) fn collect_step_filters_with_sources(
    root: &Path,
    context: StepContext<'_>,
    step: &Value,
    sources: &crate::codebase::ts_source::SourceStore,
    filters: &mut Vec<CiFilter>,
    findings: &mut Vec<RuleFinding>,
) {
    if !is_paths_filter_step(step) {
        return;
    }
    let Some(with) = step.get("with") else { return };
    let Some(raw_filters) = with.get("filters").and_then(Value::as_str) else {
        return;
    };
    let quantifier = predicate_quantifier(with);
    let Some(parsed) = parse_filters_value_with_sources(
        root,
        context.rel,
        context.job_id,
        context.step_id,
        raw_filters,
        sources,
        findings,
    ) else {
        return;
    };
    let Some(map) = parsed.as_mapping() else {
        return;
    };
    for (name, patterns) in map {
        let Some(name) = name.as_str() else { continue };
        let predicates = filter_predicates(patterns);
        let compiled = match compile_pattern_predicates(&predicates) {
            Ok(compiled) => compiled,
            Err(error) => {
                findings.push(workflow_finding(
                    context.rel,
                    format!(
                        "{}: filter `{name}` contains invalid glob: {error}",
                        context.rel
                    ),
                    Some(name.to_string()),
                ));
                continue;
            }
        };
        filters.push(CiFilter {
            workflow: context.rel.to_string(),
            name: name.to_string(),
            compiled,
            quantifier,
            workflow_paths: context.workflow_paths.clone(),
        });
    }
}

fn is_paths_filter_step(step: &Value) -> bool {
    step.get("uses")
        .and_then(Value::as_str)
        .is_some_and(|uses| uses.trim().starts_with("dorny/paths-filter"))
}

fn predicate_quantifier(with: &Value) -> PredicateQuantifier {
    match with
        .get("predicate-quantifier")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "every" => PredicateQuantifier::Every,
        _ => PredicateQuantifier::Some,
    }
}
