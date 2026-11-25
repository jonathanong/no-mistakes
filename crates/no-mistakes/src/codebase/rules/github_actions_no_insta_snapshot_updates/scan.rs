use super::{NAME, RULE_ID};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::github_actions_workflow_files::{
    env_policy::{
        lines::{locate, parse, Seg},
        scalar_text, visit_workflow_scopes,
    },
    gha_kind,
    job_lines::document::Document,
};
use crate::codebase::rules::{read_language_source, suppress_rule_findings_with_source, RuleFinding};
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path, SourceStore};
use crate::config::v2::Unanalyzable;
use serde_yaml::Mapping;
use std::path::Path;

pub(super) fn check_workflow(
    root: &Path,
    path: &Path,
    sources: &SourceStore,
    defer_suppression: bool,
    unanalyzable: Unanalyzable,
) -> Option<AnalysisDiagnostics> {
    let relative = relative_slash_path(root, path);
    if gha_kind(&relative) != Some(false) {
        return None;
    }
    let source = read_language_source(sources, path)?;
    if !defer_suppression && has_disable_file_comment(&source, RULE_ID) {
        return None;
    }
    let mut output = AnalysisDiagnostics::default();
    match sources.parse_yaml_path(path) {
        Ok(yaml) => {
            let locations = parse(&source);
            visit_workflow_scopes(&yaml, &mut |owner, path, scope| {
                check_scope(
                    &relative,
                    locations.as_ref(),
                    owner,
                    path,
                    scope,
                    unanalyzable,
                    &mut output,
                );
            });
        }
        Err(err) => output.unanalyzable(
            unanalyzable,
            finding(
                &relative,
                1,
                "yaml",
                format!(
                    "workflow YAML could not be parsed ({err}), so `{NAME}` settings cannot be checked statically. Fix the YAML syntax or suppress this file."
                ),
            ),
        ),
    }
    if !defer_suppression {
        suppress_rule_findings_with_source(&mut output.findings, &source);
        suppress_rule_findings_with_source(&mut output.advisories, &source);
    }
    Some(output)
}

fn check_scope(
    relative: &str,
    locations: Option<&Document>,
    owner: &Mapping,
    path: &[Seg],
    scope: &str,
    unanalyzable: Unanalyzable,
    output: &mut AnalysisDiagnostics,
) {
    let Some(raw_env) = owner.get("env") else {
        return;
    };
    if raw_env.is_null() {
        return;
    }
    let Some(env) = raw_env.as_mapping() else {
        let mut at = path.to_vec();
        at.push(Seg::Key("env".into()));
        let detail = scalar_text(raw_env).unwrap_or_else(|| "a non-mapping value".into());
        output.unanalyzable(
            unanalyzable,
            finding(
                relative,
                locate(locations, &at),
                "env",
                format!(
                    "{scope} sets `env` to `{detail}`, which cannot be checked statically for `{NAME}`. Write a literal `env:` mapping so the rule can verify it, or suppress this line."
                ),
            ),
        );
        return;
    };
    let Some(value) = env.get(NAME) else {
        return;
    };
    let mut at = path.to_vec();
    at.extend([Seg::Key("env".into()), Seg::Key(NAME.into())]);
    let line = locate(locations, &at);
    match scalar_text(value) {
        None => output.unanalyzable(
            unanalyzable,
            finding(
                relative,
                line,
                NAME,
                format!(
                    "{scope} sets `{NAME}` to a non-scalar value, which cannot be checked statically. Set `{NAME}` to a literal string such as `\"no\"`, or suppress this line."
                ),
            ),
        ),
        Some(mode) if mode.contains("${{") => output.unanalyzable(
            unanalyzable,
            finding(
                relative,
                line,
                NAME,
                format!(
                    "{scope} sets `{NAME}` to the expression `{mode}`, which cannot be checked statically. Set `{NAME}` to a literal value such as `\"no\"`, or suppress this line."
                ),
            ),
        ),
        Some(mode) if matches!(mode.as_str(), "always" | "1" | "force" | "unseen") => {
            output.findings.push(finding(
                relative,
                line,
                NAME,
                format!(
                    "{scope} sets `{NAME}` to `{mode}`, which can accept snapshots during CI. Set `{NAME}` to `\"no\"` in CI. Review and commit deliberate snapshot changes separately."
                ),
            ));
        }
        Some(_) => {}
    }
}

fn finding(relative: &str, line: usize, target: &str, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.into(),
        file: relative.into(),
        line,
        message,
        import: None,
        target: Some(target.into()),
        causes: Vec::new(),
    }
}
