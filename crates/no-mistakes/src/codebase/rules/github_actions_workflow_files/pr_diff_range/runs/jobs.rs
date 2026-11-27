use super::super::checkout::{job_effective_checkout, next_checkout_state, CheckoutState};
use super::super::options::CompiledOptions;
use super::super::pairs::{check_env_pairs, check_flag_pairs, check_with_pairs, PairOutput};
use super::super::pairs_locate::shape_unanalyzable;
use super::super::step_judge::{judge_step, Step};
use super::super::yaml_shapes::{env_of, get, guards};
use super::{literal_map, Scope};
use crate::codebase::rules::github_actions_script_expressions::StepRun;
use crate::codebase::rules::github_actions_workflow_files::run_scripts::scalar_name;
use no_mistakes_rule_kernel::finding::RuleFinding;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;

pub(super) struct JobWalk<'a, 'b> {
    pub(super) scope: &'a Scope<'b>,
    pub(super) opts: &'a CompiledOptions,
    pub(super) jobs: &'a Mapping,
    pub(super) located: &'a [StepRun],
    pub(super) workflow_env: &'a BTreeMap<String, String>,
    pub(super) pull: bool,
    pub(super) queue: bool,
    pub(super) findings: &'a mut Vec<RuleFinding>,
    pub(super) undecidable: &'a mut Vec<RuleFinding>,
}

pub(super) fn walk(input: &mut JobWalk<'_, '_>) {
    let scope = input.scope;
    let opts = input.opts;
    let jobs = input.jobs;
    let located = input.located;
    let workflow_env = input.workflow_env;
    let pull = input.pull;
    let queue = input.queue;
    let findings = &mut *input.findings;
    let undecidable = &mut *input.undecidable;
    for (id, job_raw) in jobs {
        let name = scalar_name(id);
        let Some(job) = literal_map(job_raw) else {
            if opts.strict_enabled() {
                undecidable.push(shape_unanalyzable(
                    scope.rel,
                    scope.source,
                    scope.document,
                    Some(&name),
                    None,
                    "job",
                    &format!("job `{name}`"),
                ));
            }
            continue;
        };
        let mut job_env = workflow_env.clone();
        job_env.extend(env_of(get(job, "env")));
        let job_guard = guards(get(job, "if"));
        let steps_raw = get(job, "steps").filter(|v| !v.is_null());
        let steps = match steps_raw.map(Value::as_sequence) {
            Some(Some(seq)) => Some(seq),
            Some(None) => {
                if opts.strict_enabled() {
                    undecidable.push(shape_unanalyzable(
                        scope.rel,
                        scope.source,
                        scope.document,
                        Some(&name),
                        None,
                        "steps",
                        &format!("job `{name}` `steps`"),
                    ));
                }
                None
            }
            None => None,
        };
        let job_checkout = job_effective_checkout(steps, pull, queue);
        if !opts.revision_pairs.is_empty() {
            let ctx = scope.ctx(Some(&name), None, job_checkout, None);
            let mut out = PairOutput {
                findings: &mut *findings,
                undecidable: &mut *undecidable,
            };
            check_env_pairs(
                &ctx,
                get(job, "env"),
                &job_env,
                &opts.revision_pairs,
                &mut out,
            );
            check_with_pairs(&ctx, job, &opts.revision_pairs, &mut out);
        }
        let mut checkout = CheckoutState::initial(pull, queue);
        for (index, step) in steps.into_iter().flatten().enumerate() {
            let Some(step_map) = literal_map(step) else {
                if opts.strict_enabled() {
                    undecidable.push(shape_unanalyzable(
                        scope.rel,
                        scope.source,
                        scope.document,
                        Some(&name),
                        Some(index + 1),
                        "step",
                        &format!("job `{name}` step {}", index + 1),
                    ));
                }
                continue;
            };
            checkout = next_checkout_state(checkout, step_map, pull, queue);
            let mut env = job_env.clone();
            env.extend(env_of(get(step_map, "env")));
            let located_step = located
                .iter()
                .find(|item| item.job == name && item.step == index + 1);
            if !opts.revision_pairs.is_empty() {
                let ctx = scope.ctx(Some(&name), Some(index + 1), checkout, located_step);
                let mut out = PairOutput {
                    findings: &mut *findings,
                    undecidable: &mut *undecidable,
                };
                check_env_pairs(
                    &ctx,
                    get(step_map, "env"),
                    &env,
                    &opts.revision_pairs,
                    &mut out,
                );
                check_with_pairs(&ctx, step_map, &opts.revision_pairs, &mut out);
                if let Some(script) = get(step_map, "run").and_then(Value::as_str) {
                    check_flag_pairs(&ctx, script, &env, &opts.revision_pairs, &mut out);
                }
            }
            let Some(script) = get(step_map, "run").and_then(Value::as_str) else {
                continue;
            };
            let step = Step {
                rel: scope.rel,
                source: scope.source,
                mixed: scope.mixed,
                job: &name,
                index: index + 1,
                script,
                env,
                located: located_step,
                document: scope.document,
                guarded: job_guard || guards(get(step_map, "if")),
                checkout,
            };
            judge_step(&step, pull, queue, opts, findings, undecidable);
        }
    }
}
