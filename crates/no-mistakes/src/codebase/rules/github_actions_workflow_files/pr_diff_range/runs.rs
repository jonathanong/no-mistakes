use super::checkout::{job_effective_checkout, next_checkout_state, CheckoutState};
use super::options::CompiledOptions;
use super::pairs::{check_env_pairs, check_flag_pairs, check_with_pairs, PairContext, PairOutput};
use super::pairs_locate::shape_unanalyzable;
use super::step_judge::{judge_step, Step};
use super::yaml_shapes::{env_of, events, get, guards};
use super::RULE_ID;
use crate::codebase::rules::github_actions_script_expressions::{step_run_lines, StepRun};
use crate::codebase::rules::github_actions_workflow_files::gha_kind;
use crate::codebase::rules::github_actions_workflow_files::job_lines::document::Document;
use crate::codebase::rules::github_actions_workflow_files::run_scripts::scalar_name;
use no_mistakes_rule_kernel::finding::RuleFinding;
use serde_yaml::{Mapping, Value};
use std::cell::OnceCell;

fn literal_map(value: &Value) -> Option<&Mapping> {
    value
        .as_mapping()
        .filter(|map| !map.contains_key(Value::String("<<".to_owned())))
}

struct Scope<'a> {
    rel: &'a str,
    source: &'a str,
    mixed: bool,
    pull: bool,
    queue: bool,
    document: &'a OnceCell<Option<Document>>,
}

impl<'a> Scope<'a> {
    fn ctx(
        &self,
        job: Option<&'a str>,
        step_index: Option<usize>,
        checkout: CheckoutState,
        located: Option<&'a StepRun>,
    ) -> PairContext<'a> {
        PairContext {
            rel: self.rel,
            source: self.source,
            job,
            step_index,
            mixed: self.mixed,
            pull: self.pull,
            queue: self.queue,
            checkout,
            document: self.document,
            located,
        }
    }
}

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
    let scope = Scope { rel, source, mixed, pull, queue, document: &document };
    let mut findings = Vec::new();
    let mut undecidable = Vec::new();
    let initial_checkout = CheckoutState::initial(pull, queue);
    if !opts.revision_pairs.is_empty() {
        let ctx = scope.ctx(None, None, initial_checkout, None);
        let mut out = PairOutput { findings: &mut findings, undecidable: &mut undecidable };
        check_env_pairs(&ctx, value.get("env"), &workflow_env, &opts.revision_pairs, &mut out);
    }
    let Some(jobs_raw) = value.get("jobs").filter(|v| !v.is_null()) else {
        return (findings, undecidable);
    };
    let Some(jobs) = literal_map(jobs_raw) else {
        if opts.strict_enabled() {
            undecidable.push(shape_unanalyzable(
                rel, source, &document, None, None, "jobs", "workflow `jobs`",
            ));
        }
        return (findings, undecidable);
    };
    for (id, job_raw) in jobs {
        let name = scalar_name(id);
        let Some(job) = literal_map(job_raw) else {
            if opts.strict_enabled() {
                undecidable.push(shape_unanalyzable(
                    rel, source, &document, Some(&name), None, "job", &format!("job `{name}`"),
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
                        rel, source, &document, Some(&name), None, "steps", &format!("job `{name}` `steps`"),
                    ));
                }
                None
            }
            None => None,
        };
        let job_checkout = job_effective_checkout(steps, pull, queue);
        if !opts.revision_pairs.is_empty() {
            let ctx = scope.ctx(Some(&name), None, job_checkout, None);
            let mut out = PairOutput { findings: &mut findings, undecidable: &mut undecidable };
            check_env_pairs(&ctx, get(job, "env"), &job_env, &opts.revision_pairs, &mut out);
            check_with_pairs(&ctx, job, &opts.revision_pairs, &mut out);
        }
        let mut checkout = initial_checkout;
        for (index, step) in steps.into_iter().flatten().enumerate() {
            let Some(step_map) = literal_map(step) else {
                if opts.strict_enabled() {
                    undecidable.push(shape_unanalyzable(
                        rel, source, &document, Some(&name), Some(index + 1), "step", &format!("job `{name}` step {}", index + 1),
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
                let mut out = PairOutput { findings: &mut findings, undecidable: &mut undecidable };
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
                rel, source, mixed, job: &name, index: index + 1, script, env,
                located: located_step, document: &document,
                guarded: job_guard || guards(get(step_map, "if")), checkout,
            };
            judge_step(&step, pull, queue, opts, &mut findings, &mut undecidable);
        }
    }
    (findings, undecidable)
}
