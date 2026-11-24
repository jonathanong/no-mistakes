use super::line::{nested_input_finding, nested_input_value, nested_timeout_line, step_key_line};
use super::yaml::{
    finding, is_direct_third_party, literal_u64, mapping_get, mapping_string, step_label,
    timeout_message, uses_matches, yaml_got,
};
use super::{AnalysisDiagnostics, CompiledOptions};
use serde_yaml::{Mapping, Value};

pub(super) fn check_workflow_step(
    rel: &str,
    source: &str,
    job_id: &str,
    step: &Mapping,
    index: usize,
    opts: &CompiledOptions,
    diagnostics: &mut AnalysisDiagnostics,
) {
    let Some(uses) = mapping_string(step, "uses") else {
        return;
    };
    if !uses_matches(&uses, &opts.uses) {
        return;
    }
    let label = step_label(step, index);
    let context = StepContext {
        rel,
        source,
        job_id,
        label: &label,
        index,
    };
    let direct = is_direct_third_party(&uses, &opts.uses);
    let compare = direct && opts.nested_below_step_timeout && !opts.nested_input.is_empty();
    let raw_minutes = mapping_get(step, "timeout-minutes");
    let minutes = raw_minutes.and_then(literal_u64);
    let raw_seconds = direct
        .then(|| nested_input_value(step, &opts.nested_input))
        .flatten();
    let seconds = raw_seconds.filter(|_| compare).and_then(whole_seconds);
    if let Some(expected) = opts
        .step_timeout_minutes
        .filter(|expected| minutes != Some(*expected))
    {
        diagnostics.findings.push(finding(
            rel,
            context.step_timeout_line(raw_minutes),
            timeout_message(rel, job_id, &label, expected, &yaml_got(raw_minutes)),
            job_id,
        ));
    }
    if direct {
        diagnostics.findings.extend(nested_input_finding(
            rel,
            source,
            job_id,
            &label,
            raw_seconds,
            index,
            opts,
        ));
    }
    if compare {
        context.check_deadline(
            raw_minutes,
            minutes,
            raw_seconds,
            seconds,
            opts,
            diagnostics,
        );
    }
}

struct StepContext<'a> {
    rel: &'a str,
    source: &'a str,
    job_id: &'a str,
    label: &'a str,
    index: usize,
}

impl StepContext<'_> {
    fn step_timeout_line(&self, raw_minutes: Option<&Value>) -> usize {
        let key = if raw_minutes.is_some() {
            "timeout-minutes"
        } else {
            "uses"
        };
        step_key_line(self.source, self.job_id, self.index, key)
    }

    fn check_deadline(
        &self,
        raw_minutes: Option<&Value>,
        minutes: Option<u64>,
        raw_seconds: Option<&Value>,
        seconds: Option<u128>,
        opts: &CompiledOptions,
        diagnostics: &mut AnalysisDiagnostics,
    ) {
        let Self {
            rel,
            source,
            job_id,
            label,
            index,
        } = *self;
        let input = &opts.nested_input;
        if minutes.is_none() {
            let got = yaml_got(raw_minutes);
            diagnostics.unanalyzable(
                opts.unanalyzable,
                finding(
                    rel,
                    self.step_timeout_line(raw_minutes),
                    format!("{rel}: job \"{job_id}\" step \"{label}\" needs timeout-minutes set to a nonnegative integer literal to compare with {input}. Found {got}."),
                    "unanalyzable",
                ),
            );
        }
        if seconds.is_none() {
            let got = yaml_got(raw_seconds);
            diagnostics.unanalyzable(
                opts.unanalyzable,
                finding(
                    rel,
                    nested_timeout_line(source, job_id, index, input),
                    format!("{rel}: job \"{job_id}\" step \"{label}\" needs a nonnegative numeric {input} in its with: block to compare with timeout-minutes. Found {got}."),
                    "unanalyzable",
                ),
            );
        }
        let (Some(minutes), Some(seconds)) = (minutes, seconds) else {
            return;
        };
        let cap = u128::from(minutes) * 60;
        if seconds >= cap {
            let got = yaml_got(raw_seconds);
            diagnostics.findings.push(finding(
                rel,
                nested_timeout_line(source, job_id, index, input),
                format!("{rel}: job \"{job_id}\" step \"{label}\" must set {input} below {cap} seconds (timeout-minutes: {minutes}). Found {got}."),
                job_id,
            ));
        }
    }
}

fn whole_seconds(value: &Value) -> Option<u128> {
    literal_u64(value).map(u128::from).or_else(|| {
        // An integer cap permits flooring fractional seconds without rounding the cap.
        value
            .as_f64()
            .filter(|value| value.is_finite() && *value >= 0.0)
            .map(|value| value.floor() as u128)
    })
}
