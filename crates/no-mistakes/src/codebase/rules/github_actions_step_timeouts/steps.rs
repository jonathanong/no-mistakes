use super::dynamic::{check_dynamic_only, emit_dynamic_uses, DynamicCtx, StepRef};
use super::options::{CompiledOptions, UsesMatch};
use super::yaml::{finding, has_key, is_expression, mapping_str, timeout_minutes, StepLocations};
use super::GITHUB_MAX_MINUTES;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;

struct StepScan<'a> {
    rel: &'a str,
    locations: &'a StepLocations<'a>,
    job_id: &'a str,
    opts: &'a CompiledOptions,
    diag: &'a mut AnalysisDiagnostics,
    used: &'a mut BTreeSet<String>,
}

pub(super) fn scan_steps(
    rel: &str,
    job_id: &str,
    steps: &[Value],
    opts: &CompiledOptions,
    locations: &StepLocations<'_>,
    diag: &mut AnalysisDiagnostics,
    used: &mut BTreeSet<String>,
) {
    StepScan {
        rel,
        locations,
        job_id,
        opts,
        diag,
        used,
    }
    .visit_steps(
        steps,
        &format!("{rel}#{job_id}#"),
        &locations.job_steps(job_id, steps.len()),
    );
}

impl StepScan<'_> {
    fn visit_steps(&mut self, steps: &[Value], prefix: &str, lines: &[usize]) {
        for (index, step) in steps.iter().enumerate() {
            let Some(step) = step.as_mapping() else {
                continue;
            };
            let step_id = mapping_str(step, "id")
                .map(str::to_owned)
                .unwrap_or_else(|| (index + 1).to_string());
            let key = format!("{prefix}{step_id}");
            if ["run", "uses", "wait", "wait-all"]
                .iter()
                .any(|key| has_key(step, key))
            {
                let line = lines.get(index).copied().unwrap_or(1);
                let label = mapping_str(step, "name")
                    .or_else(|| mapping_str(step, "id"))
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("step #{}", index + 1));
                self.check_timeout(&key, &label, line, step);
            }
            if let Some(nested) = step.get("parallel").and_then(Value::as_sequence) {
                let line = lines.get(index).copied().unwrap_or(1);
                let child_lines = self.locations.parallel_steps(line, nested.len());
                self.visit_steps(nested, &format!("{key}/"), &child_lines);
            }
        }
    }

    fn check_timeout(&mut self, key: &str, label: &str, line: usize, step: &Mapping) {
        let uses_match = self.opts.match_action_cap(step);
        if self.opts.only_configured_actions {
            match uses_match {
                UsesMatch::None => return,
                UsesMatch::Dynamic { raw } => {
                    let ctx = DynamicCtx {
                        rel: self.rel,
                        job_id: self.job_id,
                        opts: self.opts,
                    };
                    check_dynamic_only(
                        &ctx,
                        StepRef { key, label, line },
                        step,
                        &raw,
                        self.diag,
                        self.used,
                    );
                    return;
                }
                UsesMatch::Matched { family, cap } => {
                    self.judge_step(key, label, line, step, Some((&family, cap)), None);
                    return;
                }
            }
        }
        match uses_match {
            UsesMatch::None => self.judge_step(key, label, line, step, None, None),
            UsesMatch::Matched { family, cap } => {
                self.judge_step(key, label, line, step, Some((&family, cap)), None);
            }
            UsesMatch::Dynamic { raw } => {
                self.judge_step(key, label, line, step, None, Some(&raw));
            }
        }
    }

    fn judge_step(
        &mut self,
        key: &str,
        label: &str,
        line: usize,
        step: &Mapping,
        action_cap: Option<(&str, u64)>,
        dynamic_uses: Option<&str>,
    ) {
        let allowed = self.opts.allow.get(key);
        if allowed.is_some_and(|entry| entry.reason.trim().is_empty()) {
            self.diag.findings.push(finding(
                self.rel,
                line,
                format!("{}: allow entry `{key}` needs a reason", self.rel),
                Some(key),
            ));
        }
        let raw = step.get("timeout-minutes");
        let minutes = raw.and_then(timeout_minutes);
        match (raw, minutes, allowed) {
            (None, _, None) => self.diag.findings.push(finding(
                self.rel,
                line,
                format!("{}: job \"{}\" step \"{label}\" has no timeout-minutes and can stall until its job deadline; add a positive literal number", self.rel, self.job_id),
                Some(key),
            )),
            (None, _, Some(_)) => { self.used.insert(key.to_string()); }
            (Some(raw), None, Some(_)) if is_expression(raw) => { self.used.insert(key.to_string()); }
            (Some(_), None, _) => self.diag.findings.push(finding(
                self.rel,
                line,
                format!("{}: job \"{}\" step \"{label}\" timeout-minutes must be a positive literal number (expressions need a reasoned allow entry)", self.rel, self.job_id),
                Some(key),
            )),
            (Some(_), Some(minutes), allowed) => {
                let global_cap = self.opts.max_minutes.min(GITHUB_MAX_MINUTES);
                let (ordinary_cap, cap_source) = match action_cap {
                    Some((family, cap)) if cap <= global_cap => (cap, Some(family)),
                    _ => (global_cap, None),
                };
                let override_cap = allowed
                    .and_then(|entry| entry.max_minutes)
                    .map(|max| max.min(GITHUB_MAX_MINUTES));
                let cap = override_cap.unwrap_or(ordinary_cap);
                if override_cap.is_some_and(|max| max < ordinary_cap || (max > ordinary_cap && minutes > ordinary_cap)) {
                    self.used.insert(key.to_string());
                }
                if minutes > cap {
                    let tail = match (override_cap, cap_source) {
                        (None, Some(family)) => format!("over the {cap}-minute cap for `{family}`"),
                        _ => format!("over the {cap}-minute cap"),
                    };
                    self.diag.findings.push(finding(
                        self.rel,
                        line,
                        format!("{}: job \"{}\" step \"{label}\" timeout-minutes is {minutes}, {tail}", self.rel, self.job_id),
                        Some(key),
                    ));
                } else if let Some(raw_uses) = dynamic_uses {
                    let above_strictest = self
                        .opts
                        .min_action_cap
                        .is_some_and(|min_cap| minutes > min_cap);
                    if above_strictest && override_cap.is_some() {
                        self.used.insert(key.to_string());
                    } else if above_strictest {
                        let ctx = DynamicCtx {
                            rel: self.rel,
                            job_id: self.job_id,
                            opts: self.opts,
                        };
                        emit_dynamic_uses(&ctx, StepRef { key, label, line }, raw_uses, self.diag);
                    }
                }
            }
        }
    }
}
