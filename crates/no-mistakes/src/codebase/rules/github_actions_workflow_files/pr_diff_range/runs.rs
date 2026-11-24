use super::diff::{ranges, Form, Range};
use super::env_read;
use super::exprs::{extract, unguarded_pull_request_read, End, PULL_REQUEST_READ};
use super::shell_guard;
use super::yaml_shapes::{env_of, events, get, guards};
use super::RULE_ID;
use crate::codebase::rules::github_actions_script_expressions::{step_run_lines, StepRun};
use crate::codebase::rules::github_actions_workflow_files::gha_kind;
use crate::codebase::rules::github_actions_workflow_files::job_lines::document::Document;
use crate::codebase::rules::github_actions_workflow_files::run_scripts::scalar_name;
use crate::codebase::rules::github_actions_workflow_files::script_line::yaml_line;
use crate::codebase::rules::RuleFinding;
use serde_yaml::Value;
use std::cell::OnceCell;
use std::collections::BTreeMap;

struct Step<'a> {
    rel: &'a str,
    source: &'a str,
    mixed: bool,
    job: &'a str,
    index: usize,
    script: &'a str,
    env: BTreeMap<String, String>,
    located: Option<&'a StepRun>,
    document: &'a OnceCell<Option<Document>>,
    guarded: bool,
}

pub(super) fn check_source(rel: &str, source: &str) -> Vec<RuleFinding> {
    if gha_kind(rel) != Some(false) {
        return Vec::new();
    }
    let source = source.trim_start_matches('\u{feff}');
    let Ok(value) = serde_yaml::from_str::<Value>(source) else {
        return Vec::new();
    };
    let events = events(&value);
    let pull = events
        .iter()
        .any(|name| matches!(name.as_str(), "pull_request" | "pull_request_target"));
    let queue = events.iter().any(|name| name == "merge_group");
    let mixed = queue || events.iter().any(|name| name == "push");
    let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) else {
        return Vec::new();
    };
    if !pull && !queue {
        return Vec::new();
    }
    let located = step_run_lines(source, false);
    let workflow_env = env_of(value.get("env"));
    let document = OnceCell::new();
    let mut findings = Vec::new();
    for (id, job) in jobs {
        let Some(job) = job.as_mapping() else {
            continue;
        };
        let name = scalar_name(id);
        let mut job_env = workflow_env.clone();
        job_env.extend(env_of(get(job, "env")));
        let job_guard = guards(get(job, "if"));
        let steps = get(job, "steps").and_then(Value::as_sequence);
        for (index, step) in steps.into_iter().flatten().enumerate() {
            let Some(step) = step.as_mapping() else {
                continue;
            };
            let Some(script) = get(step, "run").and_then(Value::as_str) else {
                continue;
            };
            let mut env = job_env.clone();
            env.extend(env_of(get(step, "env")));
            let step = Step {
                rel,
                source,
                mixed,
                job: &name,
                index: index + 1,
                script,
                env,
                located: located
                    .iter()
                    .find(|item| item.job == name && item.step == index + 1),
                document: &document,
                guarded: job_guard || guards(get(step, "if")),
            };
            findings.extend(judge(&step, pull, queue));
        }
    }
    findings
}

fn judge(step: &Step, pull: bool, queue: bool) -> Vec<RuleFinding> {
    let mut out = Vec::new();
    if pull {
        let (text, exprs) = extract(step.script);
        for range in ranges(&text, &exprs, &step.env) {
            if let Some(reason) = reason(&range, step.mixed) {
                out.push(finding(step, range.line, "diff", reason));
            }
        }
    }
    if queue && !step.guarded {
        match pull_request_read(step) {
            Some(Read::Script(line)) => {
                out.push(finding(step, line, PULL_REQUEST_READ, MERGE_GROUP));
            }
            Some(Read::Env(line)) => out.push(finding_at(step, line, MERGE_GROUP)),
            None => {}
        }
    }
    out
}

const TWO_DOT: &str = "diffs the pull request base SHA against the head SHA with a two-dot or \
    two-argument range, which also lists base-branch changes made after the branch point, so \
    path filters and changed-file checks run on the wrong set. Use a three-dot diff \
    (`git diff <base>...<head>`) after fetching enough history, or `git diff HEAD^1 HEAD` on \
    the merge commit";
const PAIRED: &str = "pairs the pull request base with `github.sha` in a diff range. The two \
    values are computed at different times and mean different things on `push` and \
    `merge_group` events. Use a three-dot diff of the base and head SHAs after a deep enough \
    fetch, or `git diff HEAD^1 HEAD` on the merge commit";
const MERGE_GROUP: &str = "reads `github.event.pull_request.*` in a workflow triggered by \
    `merge_group`, where that object is empty. Use `github.event.merge_group.base_sha` and \
    `github.event.merge_group.head_sha`, or `git diff HEAD^1 HEAD`, or guard the step with \
    `github.event_name`";

fn reason(range: &Range, mixed: bool) -> Option<&'static str> {
    let ends = [range.left, range.right];
    let base = ends
        .iter()
        .any(|end| matches!(end, End::BaseSha | End::BaseRef));
    if mixed && base && ends.contains(&End::GithubSha) {
        return Some(PAIRED);
    }
    let both = ends.contains(&End::BaseSha) && ends.contains(&End::HeadSha);
    (both && range.form != Form::ThreeDot).then_some(TWO_DOT)
}

fn finding(step: &Step, script_line: usize, token: &str, reason: &str) -> RuleFinding {
    let line = yaml_line(step.located, step.script, script_line, token);
    finding_at(step, line, reason)
}

fn finding_at(step: &Step, line: usize, reason: &str) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: step.rel.to_string(),
        line,
        message: format!(
            "{}:{line}: job `{}` step {} {reason}.",
            step.rel, step.job, step.index
        ),
        import: None,
        target: Some(step.job.to_string()),
    }
}

enum Read {
    Script(usize),
    Env(usize),
}

fn pull_request_read(step: &Step) -> Option<Read> {
    if let Some(index) = unguarded_pull_request_read(step.script) {
        return Some(Read::Script(1 + step.script[..index].bytes().filter(|&b| b == b'\n').count()));
    }
    let guard = OnceCell::new();
    let (name, _) = step.env.iter().find(|(name, value)| {
        unguarded_pull_request_read(value).is_some()
            && match guard.get_or_init(|| shell_guard::excluded_on_merge_group(step.script, &step.env)) {
                Some(span) => env_read::references(&step.script[..span.start], name)
                    || env_read::references(&step.script[span.end..], name),
                None => env_read::references(step.script, name),
            }
    })?;
    let line = step
        .document
        .get_or_init(|| Document::parse(step.source))
        .as_ref()
        .and_then(|doc| env_read::line(doc, step.job, step.index - 1, name))
        .unwrap_or_else(|| step.located.map_or(1, |run| run.item_line));
    Some(Read::Env(line))
}
