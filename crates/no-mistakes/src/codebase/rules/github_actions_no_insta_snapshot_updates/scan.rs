use super::{NAME, RULE_ID};
use crate::codebase::rules::github_actions_workflow_files::{
    env_policy::{
        lines::{locate, parse, Seg},
        scalar_text, shape_detail, visit_workflow_scopes, ScopeEvent,
    },
    gha_kind,
    job_lines::document::Document,
};
use crate::codebase::rules::{source_access, RuleFinding};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use serde_yaml::Mapping;
use std::path::Path;

const UNANALYZABLE_TARGET: &str = "unanalyzable";

pub(super) fn check_workflow(
    root: &Path,
    path: &Path,
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    let relative = relative_slash_path(root, path);
    if gha_kind(&relative) != Some(false) {
        return Vec::new();
    }
    let source = match source_access::checked_source(sources, path, root, RULE_ID) {
        Ok(source) => source,
        Err(finding) => return vec![*finding],
    };
    match sources.parse_yaml_path(path) {
        Ok(yaml) => {
            let locations = parse(&source);
            let mut findings = Vec::new();
            visit_workflow_scopes(&yaml, &mut |event| match event {
                ScopeEvent::Mapping { owner, path, scope } => {
                    check_scope(&relative, locations.as_ref(), owner, path, scope, &mut findings);
                }
                ScopeEvent::Unanalyzable {
                    path, description, ..
                } => {
                    findings.push(finding(
                        &relative,
                        locate(locations.as_ref(), path),
                        UNANALYZABLE_TARGET,
                        format!(
                            "{description}, so `{NAME}` settings cannot be checked statically. Write literal workflow mappings and step sequences so the rule can verify them, or suppress this line."
                        ),
                    ));
                }
            });
            findings
        }
        Err(err) => vec![finding(
            &relative,
            err.line(),
            UNANALYZABLE_TARGET,
            format!(
                "workflow YAML could not be parsed ({err}), so `{NAME}` settings cannot be checked statically. Fix the YAML syntax or suppress this file."
            ),
        )],
    }
}

fn check_scope(
    relative: &str,
    locations: Option<&Document>,
    owner: &Mapping,
    path: &[Seg],
    scope: &str,
    findings: &mut Vec<RuleFinding>,
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
        let detail = shape_detail(raw_env, "a non-mapping value");
        findings.push(finding(
            relative,
            locate(locations, &at),
            UNANALYZABLE_TARGET,
            format!(
                "{scope} sets `env` to `{detail}`, which cannot be checked statically for `{NAME}`. Write a literal `env:` mapping so the rule can verify it, or suppress this line."
            ),
        ));
        return;
    };
    if env.contains_key("<<") {
        let mut at = path.to_vec();
        at.extend([Seg::Key("env".into()), Seg::Key("<<".into())]);
        findings.push(finding(
            relative,
            locate(locations, &at),
            UNANALYZABLE_TARGET,
            format!(
                "{scope} `env` uses YAML merge key `<<`, which cannot be checked statically for `{NAME}`. Write literal `env:` entries so the rule can verify them, or suppress this line."
            ),
        ));
    }
    let Some(value) = env.get(NAME) else {
        return;
    };
    let mut at = path.to_vec();
    at.extend([Seg::Key("env".into()), Seg::Key(NAME.into())]);
    let line = locate(locations, &at);
    match scalar_text(value) {
        None => findings.push(finding(
            relative,
            line,
            UNANALYZABLE_TARGET,
            format!(
                "{scope} sets `{NAME}` to a non-scalar value, which cannot be checked statically. Set `{NAME}` to a literal string such as `\"no\"`, or suppress this line."
            ),
        )),
        Some(mode) if mode.contains("${{") => findings.push(finding(
            relative,
            line,
            UNANALYZABLE_TARGET,
            format!(
                "{scope} sets `{NAME}` to the expression `{mode}`, which cannot be checked statically. Set `{NAME}` to a literal value such as `\"no\"`, or suppress this line."
            ),
        )),
        Some(mode) if matches!(mode.as_str(), "always" | "1" | "force" | "unseen") => {
            findings.push(finding(
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
