use super::RULE_ID;
use crate::codebase::rules::{analysis_policy::AnalysisDiagnostics, RuleFinding};
use crate::config::v2::Unanalyzable;
use serde_yaml::Value;
use std::collections::BTreeMap;

pub(super) fn check_source(
    rel: &str,
    source: &str,
    value: &Value,
    policy: Unanalyzable,
) -> AnalysisDiagnostics {
    let mut output = AnalysisDiagnostics::default();
    let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) else {
        output.unanalyzable(
            policy,
            unresolved(rel, 1, "unanalyzable", "jobs is not a mapping"),
        );
        return output;
    };
    let locations = super::locations::collect(source);
    for (job_id, job) in jobs {
        let Some(job_id) = job_id.as_str() else {
            continue;
        };
        let Some(steps) = job.get("steps").and_then(Value::as_sequence) else {
            continue;
        };
        let action_paths: Vec<_> = steps
            .iter()
            .filter_map(|step| step.get("uses")?.as_str()?.strip_prefix("./"))
            .collect();
        let mut ready: BTreeMap<&str, bool> =
            action_paths.into_iter().map(|path| (path, false)).collect();
        for (index, step) in steps.iter().enumerate() {
            let Some(uses) = step.get("uses").and_then(Value::as_str) else {
                continue;
            };
            if let Some(path) = uses.strip_prefix("./") {
                if !ready.get(path).copied().unwrap_or(false) {
                    let line = locations
                        .get(job_id)
                        .and_then(|items| items.get(&(index + 1)))
                        .copied()
                        .unwrap_or(1);
                    output.findings.push(RuleFinding { causes: Vec::new(),
                        rule: RULE_ID.to_owned(), file: rel.to_owned(), line,
                        message: format!("{rel}:{line}: job `{job_id}` uses local action `{uses}` before an unconditional root self checkout covering its path. Add actions/checkout before this step without `if` or a nonroot `path`, and include the action directory in sparse-checkout. Local actions must exist in the workspace before GitHub can load them."),
                        import: Some(uses.to_owned()), target: Some(format!("{job_id}#{}", index + 1)),
                    });
                }
            }
            if !is_checkout(uses) || step.get("if").is_some() || !root_path(step) {
                continue;
            }
            if !self_repository(step) {
                ready.values_mut().for_each(|state| *state = false);
            } else {
                for (path, state) in &mut ready {
                    if sparse_covers(step, path) {
                        *state = true;
                    }
                }
            }
        }
    }
    output
}

fn is_checkout(uses: &str) -> bool {
    uses.split_once('@').is_some_and(|(name, reference)| {
        name.eq_ignore_ascii_case("actions/checkout") && !reference.is_empty()
    })
}

fn root_path(step: &Value) -> bool {
    match step.get("with").and_then(|with| with.get("path")) {
        None => true,
        Some(Value::String(path)) => matches!(path.trim(), "." | "./"),
        _ => false,
    }
}

fn self_repository(step: &Value) -> bool {
    match step.get("with").and_then(|with| with.get("repository")) {
        None => true,
        Some(Value::String(repository)) => repository.trim() == "${{ github.repository }}",
        _ => false,
    }
}

fn sparse_covers(step: &Value, action: &str) -> bool {
    let Some(sparse) = step
        .get("with")
        .and_then(|with| with.get("sparse-checkout"))
    else {
        return true;
    };
    let Some(sparse) = sparse.as_str() else {
        return false;
    };
    if sparse.lines().any(|line| line.trim().starts_with('!')) {
        return false;
    }
    let action = crate::codebase::workflow_topology::posix_path::normalize(action);
    sparse.lines().any(|line| {
        let path = line.trim().trim_end_matches('/');
        if path.is_empty() || path.contains(['*', '?', '[', '!', '$', '\\']) {
            return false;
        }
        let path = crate::codebase::workflow_topology::posix_path::normalize(path);
        path == "."
            || action == path
            || action
                .strip_prefix(path.as_str())
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

pub(super) fn unresolved(rel: &str, line: usize, job: &str, reason: &str) -> RuleFinding {
    RuleFinding { causes: Vec::new(),
        rule: RULE_ID.to_owned(), file: rel.to_owned(), line,
        message: format!("{rel}:{line}: cannot determine local action checkout order ({reason}). Use a static workflow jobs mapping so local action availability can be checked."),
        import: None, target: Some(job.to_owned()),
    }
}
