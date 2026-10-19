use super::permissions::{job_proof, triggers_workflow_call, Proof};
use super::uses::is_checkout_uses;
use super::{gha_kind, RuleFinding, RULE_ID};
use crate::codebase::rules::github_actions_script_expressions::{step_uses_lines, StepUse};
use serde_yaml::{Mapping, Value};

struct Checkout {
    job: String,
    step: usize,
    reason: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Persist {
    IsFalse,
    NotFalse,
    Unknown,
}

pub(super) fn check_source(rel: &str, source: &str) -> Vec<RuleFinding> {
    let Some(action) = gha_kind(rel) else {
        return Vec::new();
    };
    let source = source.trim_start_matches('\u{feff}');
    // Composite and other action files do not grant GITHUB_TOKEN. A permissions
    // or workflow_call key in an action file is not applied by GitHub.
    if action {
        return Vec::new();
    }
    // Unparseable YAML is not proof that a write token is persisted.
    let Ok(value) = serde_yaml::from_str::<Value>(source) else {
        return Vec::new();
    };
    let checkouts = workflow_checkouts(&value);
    if checkouts.iter().all(|checkout| checkout.reason.is_none()) {
        return Vec::new();
    }
    // The shared step locator keeps compact sequences and block scalars. Flow
    // style has no `uses` line to name, so those findings stay on line 1.
    let locations = step_uses_lines(source);
    checkouts
        .into_iter()
        .filter_map(|checkout| {
            let reason = checkout.reason?;
            let line = located_line(&locations, &checkout.job, checkout.step);
            Some(finding(rel, line, &checkout.job, checkout.step, &reason))
        })
        .collect()
}

fn located_line(locations: &[StepUse], job: &str, step: usize) -> usize {
    locations
        .iter()
        .find(|item| item.step == step && item.job == job)
        .map(|item| item.line)
        .unwrap_or(1)
}

fn workflow_checkouts(value: &Value) -> Vec<Checkout> {
    let Some(doc) = value.as_mapping() else {
        return Vec::new();
    };
    let callable = triggers_workflow_call(doc);
    let workflow_permissions = mapping_get(doc, "permissions");
    let Some(jobs) = mapping_get(doc, "jobs").and_then(Value::as_mapping) else {
        return Vec::new();
    };
    let mut checkouts = Vec::new();
    for (job_id, job) in jobs {
        let Some(job) = job.as_mapping() else {
            continue;
        };
        let job_name = scalar_name(job_id);
        let proof = job_proof(job, workflow_permissions, callable);
        collect_steps(job, &job_name, &proof, &mut checkouts);
    }
    checkouts
}

fn collect_steps(job: &Mapping, job_name: &str, proof: &Proof, checkouts: &mut Vec<Checkout>) {
    let Some(steps) = mapping_get(job, "steps").and_then(Value::as_sequence) else {
        return;
    };
    for (index, step) in steps.iter().enumerate() {
        let Some(step) = step.as_mapping() else {
            continue;
        };
        let Some(uses) = mapping_get(step, "uses").and_then(Value::as_str) else {
            continue;
        };
        if !is_checkout_uses(uses) {
            continue;
        }
        let reason = flagged(proof, step).map(str::to_string);
        checkouts.push(Checkout {
            job: job_name.to_string(),
            step: index + 1,
            reason,
        });
    }
}

fn flagged<'a>(proof: &'a Proof, step: &Mapping) -> Option<&'a str> {
    let Proof::Write(reason) = proof else {
        return None;
    };
    if persist(step) != Persist::NotFalse || token_set(step) {
        return None;
    }
    Some(reason)
}

/// An explicit `token` is not the job `GITHUB_TOKEN`. Do not prove it can write.
fn token_set(step: &Mapping) -> bool {
    let Some(Value::Mapping(map)) = mapping_get(step, "with") else {
        return false;
    };
    mapping_get(map, "token").is_some()
}

fn persist(step: &Mapping) -> Persist {
    let Some(with) = mapping_get(step, "with") else {
        return Persist::NotFalse;
    };
    match with {
        Value::Null => Persist::NotFalse,
        Value::Mapping(map) => match mapping_get(map, "persist-credentials") {
            None => Persist::NotFalse,
            Some(value) => classify_persist(value),
        },
        _ => Persist::Unknown,
    }
}

fn classify_persist(value: &Value) -> Persist {
    match value {
        Value::Bool(false) => Persist::IsFalse,
        Value::Bool(true) => Persist::NotFalse,
        Value::String(text) => classify_persist_str(text),
        _ => Persist::Unknown,
    }
}

fn classify_persist_str(text: &str) -> Persist {
    match text.trim().to_ascii_lowercase().as_str() {
        "true" => Persist::NotFalse,
        "false" => Persist::IsFalse,
        _ => Persist::Unknown,
    }
}

fn scalar_name(key: &Value) -> String {
    match key {
        Value::String(name) if !name.is_empty() => name.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(value) => value.to_string(),
        _ => "job".to_string(),
    }
}

fn mapping_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_owned()))
}

fn finding(rel: &str, line: usize, job: &str, step: usize, reason: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message: format!(
            "{rel}:{line}: job `{job}` step {step} uses actions/checkout while the GITHUB_TOKEN \
             can write ({reason}), but persist-credentials is not false. actions/checkout writes \
             that token into the git credential store unless with.persist-credentials is false."
        ),
        import: None,
        target: Some(job.to_string()),
    }
}
