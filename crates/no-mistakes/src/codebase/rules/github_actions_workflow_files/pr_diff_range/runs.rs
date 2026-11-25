use super::checkout::{job_effective_checkout, next_checkout_state, CheckoutState};
use super::options::CompiledOptions;
use super::pairs::{check_env_pairs, check_flag_pairs, check_with_pairs, PairContext, PairOutput};
use super::step_judge::{judge_step, Step};
use super::yaml_shapes::{env_of, events, get, guards};
use super::RULE_ID;
use crate::codebase::rules::github_actions_script_expressions::step_run_lines;
use crate::codebase::rules::github_actions_workflow_files::gha_kind;
use crate::codebase::rules::github_actions_workflow_files::run_scripts::scalar_name;
use crate::codebase::rules::RuleFinding;
use serde_yaml::Value;
use std::cell::OnceCell;

pub(super) fn check_source_with_options(
    rel: &str,
    source: &str,
    opts: &CompiledOptions,
) -> (Vec<RuleFinding>, Vec<RuleFinding>) {
    if gha_kind(rel) != Some(false) {
        return (Vec::new(), Vec::new());
    }
    let source = source.trim_start_matches('\u{feff}');
    let value = match serde_yaml::from_str::<Value>(source) {
        Ok(value) => value,
        Err(error) => {
            if !opts.strict_enabled() {
                return (Vec::new(), Vec::new());
            }
            let line = error.location().map_or(1, |loc| loc.line());
            let note = RuleFinding {
                causes: Vec::new(),
                rule: RULE_ID.to_string(),
                file: rel.to_string(),
                line,
                message: format!(
                    "{rel}:{line}: workflow YAML could not be parsed ({error}), so PR revision pairs cannot be verified; repair the workflow YAML or suppress this file."
                ),
                import: None,
                target: Some("unanalyzable".to_string()),
            };
            return (Vec::new(), vec![note]);
        }
    };
    let events = events(&value);
    let pull = events
        .iter()
        .any(|name| matches!(name.as_str(), "pull_request" | "pull_request_target"));
    let queue = events.iter().any(|name| name == "merge_group");
    let mixed = queue || events.iter().any(|name| name == "push");
    if !pull && !queue {
        return (Vec::new(), Vec::new());
    }
    let located = step_run_lines(source, false);
    let workflow_env = env_of(value.get("env"));
    let document = OnceCell::new();
    let mut findings = Vec::new();
    let mut undecidable = Vec::new();
    let initial_checkout = CheckoutState::initial(pull, queue);
    if !opts.revision_pairs.is_empty() {
        let ctx = PairContext {
            rel,
            source,
            job: None,
            step_index: None,
            mixed,
            pull,
            queue,
            checkout: initial_checkout,
            document: &document,
            located: None,
        };
        let mut out = PairOutput {
            findings: &mut findings,
            undecidable: &mut undecidable,
        };
        check_env_pairs(&ctx, value.get("env"), &workflow_env, &opts.revision_pairs, &mut out);
    }
    let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) else {
        return (findings, undecidable);
    };
    for (id, job) in jobs {
        let Some(job) = job.as_mapping() else {
            continue;
        };
        let name = scalar_name(id);
        let mut job_env = workflow_env.clone();
        job_env.extend(env_of(get(job, "env")));
        let job_guard = guards(get(job, "if"));
        let steps = get(job, "steps").and_then(Value::as_sequence);
        let job_checkout = job_effective_checkout(steps, pull, queue);
        if !opts.revision_pairs.is_empty() {
            let ctx = PairContext {
                rel,
                source,
                job: Some(&name),
                step_index: None,
                mixed,
                pull,
                queue,
                checkout: job_checkout,
                document: &document,
                located: None,
            };
            let mut out = PairOutput {
                findings: &mut findings,
                undecidable: &mut undecidable,
            };
            check_env_pairs(&ctx, get(job, "env"), &job_env, &opts.revision_pairs, &mut out);
            check_with_pairs(&ctx, job, &opts.revision_pairs, &mut out);
        }
        let mut checkout = initial_checkout;
        for (index, step) in steps.into_iter().flatten().enumerate() {
            let Some(step_map) = step.as_mapping() else {
                continue;
            };
            checkout = next_checkout_state(checkout, step_map, pull, queue);
            let mut env = job_env.clone();
            env.extend(env_of(get(step_map, "env")));
            let located_step = located
                .iter()
                .find(|item| item.job == name && item.step == index + 1);
            if !opts.revision_pairs.is_empty() {
                let ctx = PairContext {
                    rel,
                    source,
                    job: Some(&name),
                    step_index: Some(index + 1),
                    mixed,
                    pull,
                    queue,
                    checkout,
                    document: &document,
                    located: located_step,
                };
                let mut out = PairOutput {
                    findings: &mut findings,
                    undecidable: &mut undecidable,
                };
                check_env_pairs(&ctx, get(step_map, "env"), &env, &opts.revision_pairs, &mut out);
                check_with_pairs(&ctx, step_map, &opts.revision_pairs, &mut out);
                if let Some(script) = get(step_map, "run").and_then(Value::as_str) {
                    check_flag_pairs(&ctx, script, &env, &opts.revision_pairs, &mut out);
                }
            }
            let Some(script) = get(step_map, "run").and_then(Value::as_str) else {
                continue;
            };
            let step = Step {
                rel,
                source,
                mixed,
                job: &name,
                index: index + 1,
                script,
                env,
                located: located_step,
                document: &document,
                guarded: job_guard || guards(get(step_map, "if")),
                checkout,
            };
            judge_step(&step, pull, queue, opts, &mut findings, &mut undecidable);
        }
    }
    (findings, undecidable)
}
