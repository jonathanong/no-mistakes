use super::yaml::{finding, has_key, is_expression, mapping_str, step_line_path, timeout_minutes};
use super::{CompiledOptions, RuleFinding, GITHUB_MAX_MINUTES};
use serde_yaml::Value;
use std::collections::BTreeSet;

struct StepScan<'a> {
    rel: &'a str,
    source: &'a str,
    job_id: &'a str,
    opts: &'a CompiledOptions,
    findings: &'a mut Vec<RuleFinding>,
    used: &'a mut BTreeSet<String>,
}

pub(super) fn scan_steps(
    rel: &str,
    source: &str,
    job_id: &str,
    steps: &[Value],
    opts: &CompiledOptions,
    findings: &mut Vec<RuleFinding>,
    used: &mut BTreeSet<String>,
) {
    StepScan {
        rel,
        source,
        job_id,
        opts,
        findings,
        used,
    }
    .visit_steps(steps, &format!("{rel}#{job_id}#"), &mut Vec::new());
}

impl StepScan<'_> {
    fn visit_steps(&mut self, steps: &[Value], prefix: &str, path: &mut Vec<usize>) {
        for (index, step) in steps.iter().enumerate() {
            let Some(step) = step.as_mapping() else {
                continue;
            };
            path.push(index);
            let step_id = mapping_str(step, "id")
                .map(str::to_owned)
                .unwrap_or_else(|| (index + 1).to_string());
            let key = format!("{prefix}{step_id}");
            if ["run", "uses", "wait", "wait-all"]
                .iter()
                .any(|key| has_key(step, key))
            {
                let line = step_line_path(self.source, self.job_id, path).unwrap_or(1);
                let label = mapping_str(step, "name")
                    .or_else(|| mapping_str(step, "id"))
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("step #{}", index + 1));
                self.check_timeout(&key, &label, line, step);
            }
            if let Some(nested) = step.get("parallel").and_then(Value::as_sequence) {
                self.visit_steps(nested, &format!("{key}/"), path);
            }
            path.pop();
        }
    }

    fn check_timeout(&mut self, key: &str, label: &str, line: usize, step: &serde_yaml::Mapping) {
        let allowed = self.opts.allow.get(key);
        if allowed.is_some_and(|entry| entry.reason.trim().is_empty()) {
            self.findings.push(finding(
                self.rel,
                line,
                format!("{}: allow entry `{key}` needs a reason", self.rel),
                Some(key),
            ));
        }
        let raw = step.get("timeout-minutes");
        let minutes = raw.and_then(timeout_minutes);
        match (raw, minutes, allowed) {
            (None, _, None) => self.findings.push(finding(self.rel, line,
                format!("{}: job \"{}\" step \"{label}\" has no timeout-minutes and can stall until its job deadline; add a positive literal number", self.rel, self.job_id), Some(key))),
            (None, _, Some(_)) => { self.used.insert(key.to_string()); }
            (Some(raw), None, Some(_)) if is_expression(raw) => { self.used.insert(key.to_string()); }
            (Some(_), None, _) => self.findings.push(finding(self.rel, line,
                format!("{}: job \"{}\" step \"{label}\" timeout-minutes must be a positive literal number (expressions need a reasoned allow entry)", self.rel, self.job_id), Some(key))),
            (Some(_), Some(minutes), allowed) => {
                let ordinary_cap = self.opts.max_minutes.min(GITHUB_MAX_MINUTES);
                let override_cap = allowed.and_then(|entry| entry.max_minutes).map(|max| max.min(GITHUB_MAX_MINUTES));
                let cap = override_cap.unwrap_or(ordinary_cap);
                if override_cap.is_some_and(|max| max < ordinary_cap || (max > ordinary_cap && minutes > ordinary_cap)) {
                    self.used.insert(key.to_string());
                }
                if minutes > cap {
                    self.findings.push(finding(self.rel, line,
                        format!("{}: job \"{}\" step \"{label}\" timeout-minutes is {minutes}, over the {cap}-minute cap", self.rel, self.job_id), Some(key)));
                }
            }
        }
    }
}
