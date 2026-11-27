use super::super::job_lines::document::Document;
use super::lines::{locate, parse, Seg};
use super::scopes::{shape_detail, visit_action_scopes, visit_workflow_scopes, ScopeEvent};
use super::{scalar_text, Policy, RULE_ID};
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::rules::{analysis_policy::AnalysisDiagnostics, source_access};
use crate::codebase::ts_source::SourceStore;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::Unanalyzable;
use serde_yaml::{Mapping, Value};
use std::path::Path;

pub(super) fn check_file(
    root: &Path,
    path: &Path,
    policy: &Policy,
    sources: &SourceStore,
    defer_suppression: bool,
    unanalyzable: Unanalyzable,
) -> Result<AnalysisDiagnostics, Box<RuleFinding>> {
    let source = sources
        .read_source_path(path)
        .map_err(|error| Box::new(source_access::read_failure(root, path, RULE_ID, &error)))?;
    if !defer_suppression && crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
    {
        return Ok(AnalysisDiagnostics::default());
    }
    let value = sources.parse_loaded_yaml(path, &source, false).map_err(|error| {
        let mut finding = source_access::unanalyzable_source(
            root,
            path,
            RULE_ID,
            format_args!("YAML could not be parsed ({error})"),
        );
        finding.line = error.line();
        Box::new(finding)
    })?;
    let rel = relative_slash_path(root, path);
    let (findings, undecidable) = check_parsed(&rel, &source, &value, policy);
    let mut output = AnalysisDiagnostics {
        findings,
        ..Default::default()
    };
    for finding in undecidable {
        output.unanalyzable(unanalyzable, finding);
    }
    if !defer_suppression {
        super::super::super::suppress_rule_findings_with_source(&mut output.findings, &source);
        super::super::super::suppress_rule_findings_with_source(&mut output.advisories, &source);
    }
    Ok(output)
}

#[derive(Default)]
struct Out {
    findings: Vec<RuleFinding>,
    undecidable: Vec<RuleFinding>,
}

struct Ctx<'a> {
    rel: &'a str,
    doc: Option<&'a Document>,
    policy: &'a Policy,
}

pub(super) fn check_parsed(
    rel: &str,
    source: &str,
    value: &Value,
    policy: &Policy,
) -> (Vec<RuleFinding>, Vec<RuleFinding>) {
    let parsed = parse(source);
    let ctx = Ctx {
        rel,
        doc: parsed.as_ref(),
        policy,
    };
    let mut out = Out::default();
    let mut visit = |event: ScopeEvent<'_>| match event {
        ScopeEvent::Mapping { owner, path, scope } => {
            judge(&ctx, owner, path, scope, &mut out);
        }
        ScopeEvent::Unanalyzable {
            path,
            target,
            description,
        } => {
            out.undecidable.push(RuleFinding {
                causes: Vec::new(),
                rule: RULE_ID.to_string(),
                file: rel.to_string(),
                line: locate(ctx.doc, path),
                message: format!(
                    "{rel}: {description}, which cannot be checked statically. Write literal workflow mappings and step sequences so the env policy can verify them, or suppress this line."
                ),
                import: None,
                target: Some(target.to_string()),
            });
        }
    };
    visit_workflow_scopes(value, &mut visit);
    visit_action_scopes(value, &mut visit);
    (out.findings, out.undecidable)
}

fn judge(ctx: &Ctx<'_>, owner: &Mapping, path: &[Seg], scope: &str, out: &mut Out) {
    let Some(raw_env) = owner.get("env") else {
        return;
    };
    if raw_env.is_null() {
        return;
    }
    let rel = ctx.rel;
    let Some(env) = raw_env.as_mapping() else {
        let mut at = path.to_vec();
        at.push(Seg::Key("env".into()));
        let detail = shape_detail(raw_env, "a non-mapping value");
        out.undecidable.push(RuleFinding {
            causes: Vec::new(),
            rule: RULE_ID.to_string(),
            file: rel.to_string(),
            line: locate(ctx.doc, &at),
            message: format!(
                "{rel}: {scope} sets `env` to `{detail}`, which cannot be checked statically. Write a literal `env:` mapping so the env policy can verify it, or suppress this line."
            ),
            import: None,
            target: Some("env".to_string()),
        });
        return;
    };
    if env.contains_key("<<") {
        let mut at = path.to_vec();
        at.extend([Seg::Key("env".into()), Seg::Key("<<".into())]);
        out.undecidable.push(RuleFinding {
            causes: Vec::new(),
            rule: RULE_ID.to_string(),
            file: rel.to_string(),
            line: locate(ctx.doc, &at),
            message: format!(
                "{rel}: {scope} `env` uses YAML merge key `<<`, which cannot be checked statically. Write literal `env:` entries so the env policy can verify them, or suppress this line."
            ),
            import: None,
            target: Some("env".to_string()),
        });
    }
    for (key, value) in env {
        let Some(name) = key.as_str() else {
            continue;
        };
        let mut undecidable = false;
        let message = if ctx.policy.forbidden.iter().any(|f| f == name) {
            format!(
                "{rel}: {scope} sets forbidden env `{name}`. Remove `{name}` from `env:`. The env policy forbids it because it changes tool or shell behavior."
            )
        } else if let Some(expected) = ctx.policy.required.get(name) {
            let got = scalar_text(value);
            if got.as_deref().is_some_and(|text| text.contains("${{")) {
                undecidable = true;
                format!(
                    "{rel}: {scope} sets env `{name}` to the expression `{}`, which cannot be checked statically. Write the literal `{expected}` so the env policy can verify it, or suppress this line if the expression always resolves to it.",
                    got.unwrap_or_default()
                )
            } else if got.as_deref() == Some(expected.as_str()) {
                continue;
            } else {
                let got = got.unwrap_or_else(|| "a non-scalar value".into());
                format!(
                    "{rel}: {scope} sets env `{name}` to `{got}`. Set `{name}` to `{expected}`. The env policy requires this value wherever it is set."
                )
            }
        } else {
            continue;
        };
        let mut at = path.to_vec();
        at.push(Seg::Key("env".into()));
        at.push(Seg::Key(name.into()));
        let target = if undecidable {
            &mut out.undecidable
        } else {
            &mut out.findings
        };
        target.push(RuleFinding {
            causes: Vec::new(),
            rule: RULE_ID.to_string(),
            file: rel.to_string(),
            line: locate(ctx.doc, &at),
            message,
            import: None,
            target: Some(name.to_string()),
        });
    }
}
