use super::RULE_ID;
use crate::codebase::rules::github_actions_workflow_files::job_lines::document::Document;
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use serde_yaml::Value;
use std::path::Path;

pub(super) fn check_file(rel: &str, path: &Path, sources: &SourceStore) -> Vec<RuleFinding> {
    let value = match sources.parse_yaml_path(path) {
        Ok(value) => value,
        Err(error) => {
            return vec![unanalyzable(rel, 1, format!("({error})"))];
        }
    };
    let Some(root_map) = value.as_mapping() else {
        return vec![unanalyzable(
            rel,
            1,
            "(workflow root is not a YAML mapping)".into(),
        )];
    };
    let Some(triggers) = root_map.get("on") else {
        return if has_opaque_key(root_map) {
            vec![unanalyzable(
                rel,
                1,
                "(unrecognised or merged key in workflow root)".into(),
            )]
        } else {
            Vec::new()
        };
    };
    let doc = sources
        .read_path(path)
        .ok()
        .and_then(|source| Document::parse(&source));
    match triggers {
        Value::String(_) => Vec::new(),
        Value::Sequence(items) => {
            if items.iter().all(|item| matches!(item, Value::String(_))) {
                Vec::new()
            } else {
                vec![unanalyzable(
                    rel,
                    on_line(doc.as_ref()),
                    "(unrecognised `on` sequence entry)".into(),
                )]
            }
        }
        Value::Mapping(on_map) => check_on_mapping(rel, on_map, doc.as_ref()),
        _ => vec![unanalyzable(
            rel,
            on_line(doc.as_ref()),
            "(unrecognised `on` trigger shape)".into(),
        )],
    }
}

fn check_on_mapping(
    rel: &str,
    on_map: &serde_yaml::Mapping,
    doc: Option<&Document>,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    if has_opaque_key(on_map) {
        findings.push(unanalyzable(
            rel,
            on_line(doc),
            "(unrecognised or merged key in `on` mapping)".into(),
        ));
    }
    for event in ["pull_request", "pull_request_target"] {
        let Some(event_value) = on_map.get(event) else {
            continue;
        };
        match event_value {
            Value::Null => {}
            Value::Mapping(event_map) => {
                for key in ["paths", "paths-ignore"] {
                    if event_map.contains_key(key) {
                        let line = filter_line(doc, event, key);
                        let target = format!("on.{event}.{key}");
                        findings.push(finding(
                            rel,
                            line,
                            &target,
                            format!(
                                "{rel}:{line}: remove the workflow-level path filter `{target}` from the required PR trigger. Select work inside the workflow and keep the required gate reporting on every PR."
                            ),
                        ));
                    }
                }
                if has_opaque_key(event_map) {
                    findings.push(unanalyzable(
                        rel,
                        event_line(doc, event),
                        format!("(unrecognised or merged key in `on.{event}`)"),
                    ));
                }
            }
            _ => findings.push(unanalyzable(
                rel,
                event_line(doc, event),
                format!("(unrecognised `on.{event}` trigger shape)"),
            )),
        }
    }
    findings
}

fn has_opaque_key(map: &serde_yaml::Mapping) -> bool {
    map.keys()
        .any(|key| !matches!(key, Value::String(name) if name != "<<"))
}

fn trigger_node(doc: &Document) -> Option<usize> {
    doc.field(*doc.roots().first()?, "on")
}

fn on_line(doc: Option<&Document>) -> usize {
    doc.and_then(|doc| doc.field_line(*doc.roots().first()?, "on"))
        .unwrap_or(1)
}

fn event_line(doc: Option<&Document>, event: &str) -> usize {
    doc.and_then(|doc| {
        let trigger = trigger_node(doc)?;
        if doc.is_alias(trigger) {
            return doc.node_line(trigger);
        }
        doc.field_line(trigger, event)
    })
    .unwrap_or(1)
}

fn filter_line(doc: Option<&Document>, event: &str, key: &str) -> usize {
    doc.and_then(|doc| {
        let trigger = trigger_node(doc)?;
        if doc.is_alias(trigger) {
            return doc.node_line(trigger);
        }
        let event_node = doc.field(trigger, event)?;
        if doc.is_alias(event_node) {
            return doc.node_line(event_node);
        }
        doc.field_line(event_node, key)
    })
    .unwrap_or(1)
}

fn unanalyzable(rel: &str, line: usize, detail: String) -> RuleFinding {
    finding(
        rel,
        line,
        "unanalyzable",
        format!(
            "Cannot inspect PR trigger filters for {rel} {detail}. Repair the workflow YAML and run the check again."
        ),
    )
}

fn finding(file: &str, line: usize, target: &str, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.into(),
        file: file.into(),
        line,
        message,
        import: None,
        target: Some(target.into()),
        causes: Vec::new(),
    }
}
