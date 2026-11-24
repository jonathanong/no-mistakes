use super::lines::{locate, Seg};
use super::{scalar_text, Policy, RuleFinding, RULE_ID};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::Unanalyzable;
use serde_yaml::{Mapping, Value};
use std::path::Path;

pub(super) fn check_file(
    root: &Path,
    path: &Path,
    policy: &Policy,
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
    unanalyzable: Unanalyzable,
) -> AnalysisDiagnostics {
    let mut output = AnalysisDiagnostics::default();
    let rel = relative_slash_path(root, path);
    let Some(source) = super::super::super::read_language_source(sources, path) else {
        return output;
    };
    if !defer_suppression && crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
    {
        return output;
    }
    if let Ok(value) = serde_yaml::from_str::<Value>(&source) {
        let (findings, undecidable) = check_parsed(&rel, &source, &value, policy);
        output.findings = findings;
        for finding in undecidable {
            output.unanalyzable(unanalyzable, finding);
        }
    }
    if !defer_suppression {
        super::super::super::suppress_rule_findings_with_source(&mut output.findings, &source);
        super::super::super::suppress_rule_findings_with_source(&mut output.advisories, &source);
    }
    output
}

#[derive(Default)]
struct Out {
    findings: Vec<RuleFinding>,
    undecidable: Vec<RuleFinding>,
}

struct Ctx<'a> {
    rel: &'a str,
    source: &'a str,
    policy: &'a Policy,
}

pub(super) fn check_parsed(
    rel: &str,
    source: &str,
    value: &Value,
    policy: &Policy,
) -> (Vec<RuleFinding>, Vec<RuleFinding>) {
    let ctx = Ctx {
        rel,
        source,
        policy,
    };
    let mut out = Out::default();
    if let Some(top) = value.as_mapping() {
        judge(&ctx, top, &[], "workflow", &mut out);
    }
    if let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) {
        for (key, job) in jobs {
            let (Some(id), Some(job)) = (key.as_str(), job.as_mapping()) else {
                continue;
            };
            let base = [Seg::Key("jobs".into()), Seg::Key(id.into())];
            let owner = format!("job \"{id}\"");
            judge(&ctx, job, &base, &owner, &mut out);
            let steps = job.get("steps").and_then(Value::as_sequence);
            let at = [base[0].clone(), base[1].clone(), Seg::Key("steps".into())];
            judge_steps(&ctx, steps, &at, &owner, &mut out);
        }
    }
    if let Some(runs) = value.get("runs").and_then(Value::as_mapping) {
        if runs.get("using").and_then(Value::as_str) == Some("composite") {
            let steps = runs.get("steps").and_then(Value::as_sequence);
            let at = [Seg::Key("runs".into()), Seg::Key("steps".into())];
            judge_steps(&ctx, steps, &at, "composite action", &mut out);
        }
    }
    (out.findings, out.undecidable)
}

fn judge_steps(
    ctx: &Ctx<'_>,
    steps: Option<&Vec<Value>>,
    path: &[Seg],
    owner: &str,
    out: &mut Out,
) {
    for (index, step) in steps.into_iter().flatten().enumerate() {
        let Some(step) = step.as_mapping() else {
            continue;
        };
        let mut at = path.to_vec();
        at.push(Seg::Idx(index));
        judge(ctx, step, &at, &format!("{owner} step {}", index + 1), out);
    }
}

fn judge(ctx: &Ctx<'_>, owner: &Mapping, path: &[Seg], scope: &str, out: &mut Out) {
    let Some(env) = owner.get("env").and_then(Value::as_mapping) else {
        return;
    };
    let rel = ctx.rel;
    for (key, value) in env {
        let Some(name) = key.as_str() else {
            continue;
        };
        let mut undecidable = false;
        let message = if ctx.policy.forbidden.iter().any(|f| f == name) {
            format!(
                "{rel}: {scope} sets forbidden env `{name}`. Remove `{name}` from `env:`. The env policy forbids it because it changes tool or shell behavior."
            )
        } else if let Some(expected) = ctx.policy.required.get(name) {
            let got = scalar_text(value);
            if got.as_deref().is_some_and(|text| text.contains("${{")) {
                undecidable = true;
                format!(
                    "{rel}: {scope} sets env `{name}` to the expression `{}`, which cannot be checked statically. Write the literal `{expected}` so the env policy can verify it, or suppress this line if the expression always resolves to it.",
                    got.unwrap_or_default()
                )
            } else if got.as_deref() == Some(expected.as_str()) {
                continue;
            } else {
                let got = got.unwrap_or_else(|| "a non-scalar value".into());
                format!(
                    "{rel}: {scope} sets env `{name}` to `{got}`. Set `{name}` to `{expected}`. The env policy requires this value wherever it is set."
                )
            }
        } else {
            continue;
        };
        let mut at = path.to_vec();
        at.push(Seg::Key("env".into()));
        at.push(Seg::Key(name.into()));
        let target = if undecidable {
            &mut out.undecidable
        } else {
            &mut out.findings
        };
        target.push(RuleFinding {
            causes: Vec::new(),
            rule: RULE_ID.to_string(),
            file: rel.to_string(),
            line: locate(ctx.source, &at),
            message,
            import: None,
            target: Some(name.to_string()),
        });
    }
}
