use super::scan::finding;
use crate::codebase::rules::github_actions_workflow_files::{
    env_policy::lines::{locate, Seg},
    job_lines::document::Document,
};
use crate::codebase::rules::RuleFinding;
use serde_yaml::{Mapping, Value};

mod commands;

pub(super) fn check(
    relative: &str,
    locations: Option<&Document>,
    workflow: &Value,
    step: &Mapping,
    path: &[Seg],
    scope: &str,
    findings: &mut Vec<RuleFinding>,
) {
    let [_, Seg::Key(job_id), _, Seg::Idx(_)] = path else {
        return;
    };
    let Some(run) = step.get("run") else {
        return;
    };
    let job = &workflow["jobs"][job_id.as_str()];
    let mut at = path.to_vec();
    at.push(Seg::Key("run".into()));
    let line = locate(locations, &at);
    for check in check_step(run, step, job, workflow) {
        let (target, message) = match check {
            commands::Check::Accept(form) => (
                form,
                format!(
                    "{scope} enables cargo-insta snapshot acceptance with `{form}`, which can accept snapshots during CI. Run `cargo insta test` without acceptance flags in CI. Review and commit deliberate snapshot changes separately."
                ),
            ),
            commands::Check::Unknown(reason) => (
                "unanalyzable",
                format!(
                    "{scope} {reason}, so cargo-insta snapshot acceptance cannot be checked statically. Use a literal command in a supported shell, or suppress this line."
                ),
            ),
        };
        findings.push(finding(relative, line, target, message));
    }
}

fn check_step(run: &Value, step: &Mapping, job: &Value, workflow: &Value) -> Vec<commands::Check> {
    use commands::Check::Unknown;

    let Some(script) = run.as_str() else {
        return vec![Unknown("sets `run` to a non-string value")];
    };
    let checks = commands::checks(script);
    if !checkable_shell(step, job, workflow)
        && (commands::mentions_insta(script) || !checks.is_empty())
    {
        return vec![Unknown("uses an unsupported or unknown shell")];
    }
    checks
}

fn checkable_shell(step: &Mapping, job: &Value, workflow: &Value) -> bool {
    let shell = step
        .get("shell")
        .or_else(|| job.get("defaults")?.get("run")?.get("shell"))
        .or_else(|| workflow.get("defaults")?.get("run")?.get("shell"));
    if let Some(shell) = shell {
        return matches!(
            shell.as_str(),
            Some("bash" | "sh" | "bash {0}" | "bash -e {0}" | "sh {0}" | "sh -e {0}")
        );
    }
    job.get("container").is_some() || job.get("runs-on").is_some_and(bourne_runner)
}

fn bourne_runner(runner: &Value) -> bool {
    if let Some(label) = runner.as_str() {
        let label = label.to_ascii_lowercase();
        return matches!(label.as_str(), "linux" | "macos")
            || ((label.starts_with("ubuntu-") || label.starts_with("macos-"))
                && !label.contains("${{"));
    }
    let labels = runner.get("labels").unwrap_or(runner);
    match labels.as_sequence() {
        Some(labels) => {
            labels.iter().all(|label| {
                label.as_str().is_some_and(|label| {
                    !label.contains("${{") && !label.to_ascii_lowercase().starts_with("windows")
                })
            }) && labels.iter().any(bourne_runner)
        }
        None => runner.get("labels").is_some_and(bourne_runner),
    }
}
