use super::inputs::{verdict, Verdict};
use super::permissions::{job_proof, triggers_workflow_call, Proof};
use super::uses::is_checkout_uses;
use super::{gha_kind, RuleFinding, RULE_ID};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::github_actions_script_expressions::{step_uses_lines, StepUse};
use crate::config::v2::Unanalyzable;
use serde_yaml::{Mapping, Value};

struct Checkout {
    job: String,
    step: usize,
    verdict: Verdict,
    declared_maximum: bool,
}

pub(super) fn analyze(
    rel: &str,
    source: &str,
    assume_unspecified: bool,
    policy: Unanalyzable,
) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    if gha_kind(rel) != Some(false) {
        return diagnostics;
    }
    let source = source.trim_start_matches('\u{feff}');
    let value = match serde_yaml::from_str::<Value>(source) {
        Ok(value) => value,
        Err(error) => {
            let line = error.location().map_or(1, |location| location.line());
            diagnostics.unanalyzable(
                policy,
                uncertain(
                    rel,
                    line,
                    "unknown",
                    "unknown",
                    &format!("invalid YAML: {error}"),
                ),
            );
            return diagnostics;
        }
    };
    let locations = step_uses_lines(source);
    for checkout in workflow_checkouts(&value, assume_unspecified) {
        let line = located_line(&locations, &checkout.job, checkout.step);
        match checkout.verdict {
            Verdict::Safe => {}
            Verdict::Write(reason) => diagnostics.findings.push(finding(
                rel,
                line,
                &checkout.job,
                checkout.step,
                &reason,
                checkout.declared_maximum,
            )),
            Verdict::Unknown(reason) => diagnostics.unanalyzable(
                policy,
                uncertain(rel, line, &checkout.job, &checkout.step.to_string(), reason),
            ),
        }
    }
    diagnostics
}

fn uncertain(rel: &str, line: usize, job: &str, step: &str, reason: &str) -> RuleFinding {
    let message = format!(
        "{rel}:{line}: job `{job}` step {step}: cannot analyze checkout credentials ({reason}). \
         Set with.persist-credentials to literal false, or use the default GITHUB_TOKEN with \
         literal read-only permissions; persisted write credentials let later steps modify \
         GitHub state."
    );
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message,
        import: None,
        target: Some(job.to_string()),
    }
}

fn located_line(locations: &[StepUse], job: &str, step: usize) -> usize {
    locations
        .iter()
        .find(|item| item.step == step && item.job == job)
        .map(|item| item.line)
        .unwrap_or(1)
}

fn workflow_checkouts(value: &Value, assume_unspecified: bool) -> Vec<Checkout> {
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
        let (proof, declared) = job_proof(job, workflow_permissions, assume_unspecified);
        // This labels a permitted maximum, not the caller's actual runtime grant.
        collect_steps(job, &job_name, &proof, callable && declared, &mut checkouts);
    }
    checkouts
}

fn collect_steps(
    job: &Mapping,
    job_name: &str,
    proof: &Proof,
    declared_maximum: bool,
    checkouts: &mut Vec<Checkout>,
) {
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
        let verdict = verdict(proof, step);
        checkouts.push(Checkout {
            job: job_name.to_string(),
            step: index + 1,
            verdict,
            declared_maximum,
        });
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

fn finding(
    rel: &str,
    line: usize,
    job: &str,
    step: usize,
    reason: &str,
    declared_maximum: bool,
) -> RuleFinding {
    let access = if declared_maximum {
        "the reusable workflow's declared permissions may allow GITHUB_TOKEN write access"
    } else {
        "the GITHUB_TOKEN can write"
    };
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message: format!(
            "{rel}:{line}: job `{job}` step {step} uses actions/checkout while {access} \
             ({reason}), but persist-credentials is not false. actions/checkout writes \
             that token into the git credential store unless with.persist-credentials is false."
        ),
        import: None,
        target: Some(job.to_string()),
    }
}
