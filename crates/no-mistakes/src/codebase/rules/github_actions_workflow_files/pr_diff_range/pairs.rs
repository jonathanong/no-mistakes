use super::checkout::CheckoutState;
use super::exprs::extract;
use super::options::CompiledRevisionPair;
use super::pairs_locate::{emit, env_pair_line, extract_flag_pair, field_line, with_pair_line};
use super::revisions::{classify_token, classify_value, Revision};
use super::verdicts::{judge_explicit_pair, Verdict};
use super::verdicts_helpers::{NON_MAPPING_ENV, NON_MAPPING_WITH};
use super::yaml_shapes::get;
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

fn literal_mapping(value: &Value) -> Option<&Mapping> {
    value
        .as_mapping()
        .filter(|map| !map.contains_key(Value::String("<<".to_owned())))
}

fn judge(ctx: &PairContext<'_>, base: Revision, head: Revision) -> Verdict {
    judge_explicit_pair(base, head, ctx.mixed, ctx.pull, ctx.queue, ctx.checkout)
}

pub(super) fn check_env_pairs(
    ctx: &PairContext<'_>,
    local_env: Option<&Value>,
    effective_env: &BTreeMap<String, String>,
    pairs: &[CompiledRevisionPair],
    out: &mut PairOutput<'_>,
) {
    let Some(raw) = local_env.filter(|v| !v.is_null()) else {
        return;
    };
    if !pairs
        .iter()
        .any(|p| matches!(p, CompiledRevisionPair::Env { .. }))
    {
        return;
    }
    let Some(local_map) = literal_mapping(raw) else {
        let line = field_line(ctx, "env");
        emit(ctx, line, "env", Verdict::Unanalyzable(NON_MAPPING_ENV), out);
        return;
    };
    for pair in pairs {
        let CompiledRevisionPair::Env { base, head } = pair else {
            continue;
        };
        if get(local_map, base).is_none() && get(local_map, head).is_none() {
            continue;
        }
        let (Some(base_val), Some(head_val)) = (effective_env.get(base), effective_env.get(head))
        else {
            continue;
        };
        let verdict = judge(
            ctx,
            classify_value(base_val, ctx.checkout),
            classify_value(head_val, ctx.checkout),
        );
        let line = env_pair_line(ctx, head, base);
        emit(ctx, line, &format!("env pair `{base}` / `{head}`"), verdict, out);
    }
}

pub(super) fn check_with_pairs(
    ctx: &PairContext<'_>,
    owner: &Mapping,
    pairs: &[CompiledRevisionPair],
    out: &mut PairOutput<'_>,
) {
    let Some(raw) = get(owner, "with").filter(|v| !v.is_null()) else {
        return;
    };
    if !pairs
        .iter()
        .any(|p| matches!(p, CompiledRevisionPair::Input { .. }))
    {
        return;
    }
    let Some(with_map) = literal_mapping(raw) else {
        let line = field_line(ctx, "with");
        emit(ctx, line, "with", Verdict::Unanalyzable(NON_MAPPING_WITH), out);
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
        let verdict = judge(ctx, base_rev, head_rev);
        let line = with_pair_line(ctx, head);
        emit(ctx, line, &format!("input pair `{base}` / `{head}`"), verdict, out);
    }
}

pub(super) fn check_flag_pairs(
    ctx: &PairContext<'_>,
    script: &str,
    env: &BTreeMap<String, String>,
    pairs: &[CompiledRevisionPair],
    out: &mut PairOutput<'_>,
) {
    if !pairs
        .iter()
        .any(|p| matches!(p, CompiledRevisionPair::Flag { .. }))
    {
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
            let verdict = judge(ctx, base_rev, head_rev);
            let line = yaml_line(ctx.located, script, command[0].line, base);
            emit(ctx, line, &format!("flag pair `{base}` / `{head}`"), verdict, out);
        }
    }
}
