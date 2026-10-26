use super::yaml::{finding, key_line, step_label, timeout_minutes};
use super::RuleFinding;
use serde_yaml::Value;

pub(super) fn findings(
    rel: &str,
    source: &str,
    job_id: &str,
    job: &serde_yaml::Mapping,
    job_timeout: u64,
    partner_cap: Option<u64>,
) -> Vec<RuleFinding> {
    let Some(steps) = job
        .get(Value::String("steps".to_string()))
        .and_then(Value::as_sequence)
    else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        let Some(step) = step.as_mapping() else {
            continue;
        };
        // The partner checks executable steps only; keep malformed mappings here.
        if partner_cap.is_some_and(|cap| job_timeout >= cap)
            && ["run", "uses", "wait", "wait-all"]
                .iter()
                .any(|key| step.contains_key(*key))
        {
            continue;
        }
        let Some(raw) = step.get(Value::String("timeout-minutes".to_string())) else {
            continue;
        };
        let label = step_label(step, index);
        let Some(minutes) = timeout_minutes(raw) else {
            findings.push(finding(
                rel,
                key_line(source, "timeout-minutes"),
                format!(
                    "{rel}: job \"{job_id}\" step \"{label}\" timeout-minutes has no supported literal upper bound"
                ),
                job_id,
            ));
            continue;
        };
        if minutes > job_timeout {
            findings.push(finding(
                rel,
                key_line(source, "timeout-minutes"),
                format!(
                    "{rel}: job \"{job_id}\" step \"{label}\" timeout-minutes is {minutes}, exceeding its job timeout of {job_timeout}"
                ),
                job_id,
            ));
        }
    }
    findings
}
