use super::checkout::CheckoutState;
use super::options::CompiledOptions;
use super::pairs::{check_env_pairs, PairContext, PairOutput};
use super::pairs_locate::shape_unanalyzable;
use super::yaml_shapes::{env_of, events};
use crate::codebase::rules::github_actions_script_expressions::{step_run_lines, StepRun};
use crate::codebase::rules::github_actions_workflow_files::job_lines::document::Document;
use no_mistakes_rule_kernel::finding::RuleFinding;
use serde_yaml::{Mapping, Value};
use std::cell::OnceCell;

mod jobs;

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

pub(super) fn check_parsed(
    rel: &str,
    source: &str,
    value: &Value,
    opts: &CompiledOptions,
) -> (Vec<RuleFinding>, Vec<RuleFinding>) {
    let events = events(value);
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
    let scope = Scope {
        rel,
        source,
        mixed,
        pull,
        queue,
        document: &document,
    };
    let mut findings = Vec::new();
    let mut undecidable = Vec::new();
    let initial_checkout = CheckoutState::initial(pull, queue);
    if !opts.revision_pairs.is_empty() {
        let ctx = scope.ctx(None, None, initial_checkout, None);
        let mut out = PairOutput {
            findings: &mut findings,
            undecidable: &mut undecidable,
        };
        check_env_pairs(
            &ctx,
            value.get("env"),
            &workflow_env,
            &opts.revision_pairs,
            &mut out,
        );
    }
    let Some(jobs_raw) = value.get("jobs").filter(|v| !v.is_null()) else {
        return (findings, undecidable);
    };
    let Some(jobs) = literal_map(jobs_raw) else {
        if opts.strict_enabled() {
            undecidable.push(shape_unanalyzable(
                rel,
                source,
                &document,
                None,
                None,
                "jobs",
                "workflow `jobs`",
            ));
        }
        return (findings, undecidable);
    };
    jobs::walk(&mut jobs::JobWalk {
        scope: &scope,
        opts,
        jobs,
        located: &located,
        workflow_env: &workflow_env,
        pull,
        queue,
        findings: &mut findings,
        undecidable: &mut undecidable,
    });
    (findings, undecidable)
}
