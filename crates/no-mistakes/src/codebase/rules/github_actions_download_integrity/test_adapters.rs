use super::assign::prepare_max_script;
use super::judge::{scan_script, Hit};
use super::runs::{check_workflow, Policy};
use super::verify::command_verifies;
use super::{gha_kind, words, RuleFinding};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::config::v2::Unanalyzable;

pub(super) fn expand_uppercase(script: &str) -> String {
    prepare_max_script(script).map_or_else(|| script.to_owned(), |rewrite| rewrite.text)
}

pub(super) fn download_hits(script: &str, helpers: &[String]) -> Vec<Hit> {
    scan_script(script, helpers, false, false, None)
}

pub(super) fn download_hits_low(script: &str, helpers: &[String]) -> Vec<Hit> {
    scan_script(script, helpers, true, false, None)
}

pub(super) fn check_source(
    rel: &str,
    source: &str,
    hosts: &[String],
    helpers: &[String],
) -> Vec<RuleFinding> {
    let Some(action) = gha_kind(rel) else {
        return Vec::new();
    };
    check_workflow(
        rel,
        source,
        action,
        &Policy {
            hosts,
            helpers,
            low: None,
            strict: false,
        },
    )
    .findings
}

pub(super) fn check_source_low(
    rel: &str,
    source: &str,
    hosts: &[String],
    helpers: &[String],
    policy: Unanalyzable,
) -> AnalysisDiagnostics {
    let Some(action) = gha_kind(rel) else {
        return AnalysisDiagnostics::default();
    };
    check_workflow(
        rel,
        source,
        action,
        &Policy {
            hosts,
            helpers,
            low: Some(policy),
            strict: false,
        },
    )
}

pub(super) fn step_verifies(script: &str) -> bool {
    commands(script).iter().any(command_verifies)
}

pub(super) fn commands(script: &str) -> Vec<words::Command> {
    words::parse(script).commands
}
