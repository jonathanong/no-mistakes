use super::checkout::CheckoutState;
use super::diff::{ranges, Form, Range};
use super::env_read;
use super::exprs::{extract, unguarded_pull_request_read, End, PULL_REQUEST_READ};
use super::options::CompiledOptions;
use super::revisions::classify_token;
use super::shell_guard;
use super::verdicts::{judge_diff_pair, Verdict};
use super::RULE_ID;
use crate::codebase::rules::github_actions_script_expressions::StepRun;
use crate::codebase::rules::github_actions_workflow_files::job_lines::document::Document;
use crate::codebase::rules::github_actions_workflow_files::script_line::yaml_line;
use crate::codebase::rules::RuleFinding;
use std::cell::OnceCell;
use std::collections::BTreeMap;

pub(super) struct Step<'a> {
    pub(super) rel: &'a str,
    pub(super) source: &'a str,
    pub(super) mixed: bool,
    pub(super) job: &'a str,
    pub(super) index: usize,
    pub(super) script: &'a str,
    pub(super) env: BTreeMap<String, String>,
    pub(super) located: Option<&'a StepRun>,
    pub(super) document: &'a OnceCell<Option<Document>>,
    pub(super) guarded: bool,
    pub(super) checkout: CheckoutState,
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

pub(super) fn judge_step(
    step: &Step<'_>,
    pull: bool,
    queue: bool,
    opts: &CompiledOptions,
    findings: &mut Vec<RuleFinding>,
    undecidable: &mut Vec<RuleFinding>,
) {
    if opts.require_consistent_revision_pairs && (pull || queue) {
        let (text, exprs) = extract(step.script);
        for range in ranges(&text, &exprs, &step.env) {
            let left = classify_token(&range.left_raw, &exprs, &step.env, step.checkout);
            let right = classify_token(&range.right_raw, &exprs, &step.env, step.checkout);
            match judge_diff_pair(
                left,
                right,
                range.form,
                step.mixed,
                pull,
                queue,
                step.checkout,
            ) {
                Verdict::Consistent => {}
                Verdict::Mismatch(msg) => findings.push(finding(step, range.line, "diff", msg)),
                Verdict::Unanalyzable(msg) => {
                    let mut note = finding(step, range.line, "diff", msg);
                    note.target = Some("unanalyzable".to_string());
                    undecidable.push(note);
                }
            }
        }
    } else if pull {
        let (text, exprs) = extract(step.script);
        for range in ranges(&text, &exprs, &step.env) {
            if let Some(reason) = legacy_reason(&range, step.mixed) {
                findings.push(finding(step, range.line, "diff", reason));
            }
        }
    }
    if queue && !step.guarded {
        match pull_request_read(step) {
            Some(Read::Script(line)) => {
                findings.push(finding(step, line, PULL_REQUEST_READ, MERGE_GROUP));
            }
            Some(Read::Env(line)) => findings.push(finding_at(step, line, MERGE_GROUP)),
            None => {}
        }
    }
}

fn legacy_reason(range: &Range, mixed: bool) -> Option<&'static str> {
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

fn finding(step: &Step<'_>, script_line: usize, token: &str, reason: &str) -> RuleFinding {
    let line = yaml_line(step.located, step.script, script_line, token);
    finding_at(step, line, reason)
}

fn finding_at(step: &Step<'_>, line: usize, reason: &str) -> RuleFinding {
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

fn pull_request_read(step: &Step<'_>) -> Option<Read> {
    if let Some(index) = unguarded_pull_request_read(step.script) {
        return Some(Read::Script(
            1 + step.script[..index]
                .bytes()
                .filter(|&b| b == b'\n')
                .count(),
        ));
    }
    let guard = OnceCell::new();
    let (name, _) = step.env.iter().find(|(name, value)| {
        unguarded_pull_request_read(value).is_some()
            && shell_guard::references_outside(
                step.script,
                name,
                guard.get_or_init(|| shell_guard::excluded_on_merge_group(step.script, &step.env)),
            )
    })?;
    let line = step
        .document
        .get_or_init(|| Document::parse(step.source))
        .as_ref()
        .and_then(|doc| env_read::line(doc, step.job, step.index - 1, name))
        .unwrap_or_else(|| step.located.map_or(1, |run| run.item_line));
    Some(Read::Env(line))
}
