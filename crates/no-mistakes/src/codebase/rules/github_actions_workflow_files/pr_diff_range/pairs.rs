use super::checkout::CheckoutState;
use super::exprs::extract;
use super::options::CompiledRevisionPair;
use super::pairs_locate::{env_pair_line, extract_flag_pair, with_pair_line};
use super::revisions::{classify_token, classify_value, Revision};
use super::verdicts::{judge_explicit_pair, Verdict};
use super::yaml_shapes::get;
use super::RULE_ID;
use crate::codebase::rules::github_actions_script_expressions::StepRun;
use crate::codebase::rules::github_actions_workflow_files::job_lines::document::Document;
use crate::codebase::rules::github_actions_workflow_files::script_line::yaml_line;
use crate::codebase::rules::github_actions_workflow_files::shell_words::commands;
use crate::codebase::rules::RuleFinding;
use serde_yaml::{Mapping, Value};
use std::cell::OnceCell;
use std::collections::BTreeMap;

pub(super) struct PairContext<'a> {
    pub(super) rel: &'a str,
    pub(super) source: &'a str,
    pub(super) job: Option<&'a str>,
    pub(super) step_index: Option<usize>,
    pub(super) mixed: bool,
    pub(super) pull: bool,
    pub(super) queue: bool,
    pub(super) checkout: CheckoutState,
    pub(super) document: &'a OnceCell<Option<Document>>,
    pub(super) located: Option<&'a StepRun>,
}

pub(super) struct PairOutput<'a> {
    pub(super) findings: &'a mut Vec<RuleFinding>,
    pub(super) undecidable: &'a mut Vec<RuleFinding>,
}

pub(super) fn check_env_pairs(
    ctx: &PairContext<'_>,
    local_env: Option<&Value>,
    effective_env: &BTreeMap<String, String>,
    pairs: &[CompiledRevisionPair],
    out: &mut PairOutput<'_>,
) {
    let Some(local_map) = local_env.and_then(Value::as_mapping) else {
        return;
    };
    for pair in pairs {
        let CompiledRevisionPair::Env { base, head } = pair else {
            continue;
        };
        let touched = get(local_map, base).is_some() || get(local_map, head).is_some();
        if !touched {
            continue;
        }
        let (Some(base_val), Some(head_val)) = (effective_env.get(base), effective_env.get(head))
        else {
            continue;
        };
        let base_rev = classify_value(base_val, ctx.checkout);
        let head_rev = classify_value(head_val, ctx.checkout);
        let line = env_pair_line(ctx, head, base);
        emit(
            ctx,
            line,
            &format!("env pair `{base}` / `{head}`"),
            judge_explicit_pair(
                base_rev,
                head_rev,
                ctx.mixed,
                ctx.pull,
                ctx.queue,
                ctx.checkout,
            ),
            out,
        );
    }
}

pub(super) fn check_with_pairs(
    ctx: &PairContext<'_>,
    owner: &Mapping,
    pairs: &[CompiledRevisionPair],
    out: &mut PairOutput<'_>,
) {
    let Some(with_map) = get(owner, "with").and_then(Value::as_mapping) else {
        return;
    };
    for pair in pairs {
        let CompiledRevisionPair::Input { base, head } = pair else {
            continue;
        };
        let (Some(base_raw), Some(head_raw)) = (get(with_map, base), get(with_map, head)) else {
            continue;
        };
        let base_rev = base_raw
            .as_str()
            .map_or(Revision::Unknown, |val| classify_value(val, ctx.checkout));
        let head_rev = head_raw
            .as_str()
            .map_or(Revision::Unknown, |val| classify_value(val, ctx.checkout));
        let line = with_pair_line(ctx, head, base);
        emit(
            ctx,
            line,
            &format!("input pair `{base}` / `{head}`"),
            judge_explicit_pair(
                base_rev,
                head_rev,
                ctx.mixed,
                ctx.pull,
                ctx.queue,
                ctx.checkout,
            ),
            out,
        );
    }
}

pub(super) fn check_flag_pairs(
    ctx: &PairContext<'_>,
    script: &str,
    env: &BTreeMap<String, String>,
    pairs: &[CompiledRevisionPair],
    out: &mut PairOutput<'_>,
) {
    let has_flags = pairs
        .iter()
        .any(|pair| matches!(pair, CompiledRevisionPair::Flag { .. }));
    if !has_flags {
        return;
    }
    let (text, exprs) = extract(script);
    for command in commands(&text).commands {
        for pair in pairs {
            let CompiledRevisionPair::Flag { base, head } = pair else {
                continue;
            };
            let Some((base_tok, head_tok)) = extract_flag_pair(&command, base, head) else {
                continue;
            };
            let base_rev = base_tok
                .map_or(Revision::Unknown, |tok| classify_token(tok, &exprs, env, ctx.checkout));
            let head_rev = head_tok
                .map_or(Revision::Unknown, |tok| classify_token(tok, &exprs, env, ctx.checkout));
            let line = yaml_line(ctx.located, script, command[0].line, base);
            emit(
                ctx,
                line,
                &format!("flag pair `{base}` / `{head}`"),
                judge_explicit_pair(
                    base_rev,
                    head_rev,
                    ctx.mixed,
                    ctx.pull,
                    ctx.queue,
                    ctx.checkout,
                ),
                out,
            );
        }
    }
}

fn emit(
    ctx: &PairContext<'_>,
    line: usize,
    label: &str,
    verdict: Verdict,
    out: &mut PairOutput<'_>,
) {
    let where_text = match (ctx.job, ctx.step_index) {
        (Some(job), Some(step)) => format!("job `{job}` step {step}"),
        (Some(job), None) => format!("job `{job}`"),
        (None, _) => "workflow".to_string(),
    };
    match verdict {
        Verdict::Consistent => {}
        Verdict::Mismatch(reason) => out.findings.push(RuleFinding {
            causes: Vec::new(),
            rule: RULE_ID.to_string(),
            file: ctx.rel.to_string(),
            line,
            message: format!("{}:{line}: {where_text} ({label}) {reason}.", ctx.rel),
            import: None,
            target: ctx.job.map(str::to_owned),
        }),
        Verdict::Unanalyzable(reason) => out.undecidable.push(RuleFinding {
            causes: Vec::new(),
            rule: RULE_ID.to_string(),
            file: ctx.rel.to_string(),
            line,
            message: format!("{}:{line}: {where_text} ({label}) {reason}.", ctx.rel),
            import: None,
            target: Some("unanalyzable".to_string()),
        }),
    }
}
