use super::output::proven_output_names;
use super::scan::{self, Step};
use super::text::has_fragment;
use serde_yaml::Value;

pub(super) fn proves(
    step: &Step,
    key: &str,
    lockfile: &str,
    actions: &dyn Fn(&str) -> Option<String>,
) -> bool {
    let Some(id) = trimmed_id(step) else {
        return false;
    };
    let Some(uses) = step.uses.as_deref() else {
        return false;
    };
    let Some(source) = actions(uses) else {
        return false;
    };
    let Ok(value) = serde_yaml::from_str::<Value>(&source) else {
        return false;
    };
    if value["runs"]["using"].as_str() != Some("composite") {
        return false;
    }
    let steps = match scan::analyze(&source, true, &[]) {
        scan::Document::Jobs(jobs) => jobs
            .into_iter()
            .flat_map(|job| job.steps)
            .collect::<Vec<_>>(),
        _ => return false,
    };
    let Some(outputs) = value.get("outputs").and_then(Value::as_mapping) else {
        return false;
    };
    outputs.iter().any(|(name, spec)| {
        let Some(name) = name.as_str() else {
            return false;
        };
        let Some(expr) = spec.get("value").and_then(Value::as_str) else {
            return false;
        };
        has_fragment(key, &format!("steps.{id}.outputs.{name}"))
            && step_proves(&steps, expr, lockfile)
    })
}

fn step_proves(steps: &[Step], key: &str, lockfile: &str) -> bool {
    steps.iter().any(|step| {
        let Some(id) = trimmed_id(step) else {
            return false;
        };
        let Some(run) = step.run.as_deref() else {
            return false;
        };
        proven_output_names(run, lockfile)
            .iter()
            .any(|name| has_fragment(key, &format!("steps.{id}.outputs.{name}")))
    })
}

fn trimmed_id(step: &Step) -> Option<&str> {
    step.id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
}

pub(super) fn local_action_dir(uses: &str) -> Option<&str> {
    let uses = uses.trim().trim_matches(['"', '\'']);
    let rel = uses.strip_prefix("./")?.trim_start_matches('/');
    if rel.is_empty()
        || rel
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    Some(rel)
}
