use super::options::Compiled;
use super::RULE_ID;
use crate::codebase::rules::{github_actions_workflow_files as workflows, RuleFinding};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use serde_yaml::Value;
use std::path::Path;
use workflows::job_lines::document::Document;

mod location;
mod step;
use location::{field, finding, line, StepLocation};

pub(super) fn file(
    root: &Path,
    path: &Path,
    sources: &SourceStore,
    options: &Compiled,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    let kind = workflows::gha_kind(&rel).or_else(|| {
        matches!(rel.rsplit('/').next(), Some("action.yml" | "action.yaml")).then_some(true)
    });
    let Some(composite_file) = kind else {
        return Vec::new();
    };
    if composite_file && options.caches.is_empty() {
        return Vec::new();
    }
    let source = match sources.read_path(path) {
        Ok(source) => source,
        Err(error) => {
            return workflows::unreadable_finding(&error, true, |line| {
                finding(&rel, line, "unanalyzable", "cannot read YAML source")
            })
            .into_iter()
            .collect()
        }
    };
    let value = match sources.parse_yaml_path(path) {
        Ok(value) => value,
        Err(error) => {
            return vec![finding(
                &rel,
                1,
                "unanalyzable",
                &format!("cannot parse Actions YAML ({error})"),
            )]
        }
    };
    let document = Document::parse(&source);
    let document = document.as_ref();
    if composite_file {
        return composite(&rel, &value, document, options);
    }
    let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) else {
        return vec![finding(
            &rel,
            1,
            "unanalyzable",
            "workflow jobs must be a mapping",
        )];
    };
    let jobs_node = document.and_then(|doc| doc.field(0, "jobs"));
    let mut findings = Vec::new();
    for (key, job) in jobs {
        let job_node = key.as_str().and_then(|key| field(document, jobs_node, key));
        if !job.is_mapping() {
            findings.push(finding(
                &rel,
                line(document, job_node),
                "unanalyzable",
                "workflow job must be a mapping",
            ));
        } else if let Some(steps) = job.get("steps") {
            scan_steps(
                &rel,
                steps,
                document,
                field(document, job_node, "steps"),
                options,
                true,
                &mut findings,
            );
        }
    }
    findings
}

fn composite(
    rel: &str,
    value: &Value,
    document: Option<&Document>,
    options: &Compiled,
) -> Vec<RuleFinding> {
    let runs = value.get("runs");
    let using = runs
        .and_then(|runs| runs.get("using"))
        .and_then(Value::as_str);
    let mut findings = Vec::new();
    match using {
        Some("composite") => {
            let steps_node = field(document, field(document, Some(0), "runs"), "steps");
            if let Some(steps) = runs.and_then(|runs| runs.get("steps")) {
                scan_steps(
                    rel,
                    steps,
                    document,
                    steps_node,
                    options,
                    false,
                    &mut findings,
                );
            } else {
                findings.push(finding(
                    rel,
                    line(document, steps_node),
                    "unanalyzable",
                    "composite action needs a steps sequence",
                ));
            }
        }
        Some(using) if !using.contains("${{") && !using.is_empty() => {}
        _ => findings.push(finding(
            rel,
            1,
            "unanalyzable",
            "cannot determine whether the action is composite; declare runs.using literally",
        )),
    }
    findings
}

fn scan_steps(
    rel: &str,
    steps: &Value,
    document: Option<&Document>,
    node: Option<usize>,
    options: &Compiled,
    workflow: bool,
    findings: &mut Vec<RuleFinding>,
) {
    let mut pending = vec![(steps, node)];
    while let Some((steps, node)) = pending.pop() {
        let Some(steps) = steps.as_sequence() else {
            findings.push(finding(
                rel,
                line(document, node),
                "unanalyzable",
                "workflow steps must be a sequence",
            ));
            continue;
        };
        let nodes = document
            .zip(node)
            .and_then(|(doc, node)| doc.sequence(node));
        for (index, value) in steps.iter().enumerate() {
            let node = nodes.and_then(|nodes| nodes.get(index)).copied();
            if let Some(children) = value.get("parallel") {
                pending.push((children, field(document, node, "parallel")));
            } else {
                step::scan(
                    value,
                    StepLocation {
                        rel,
                        document,
                        node,
                        workflow,
                    },
                    options,
                    findings,
                );
            }
        }
    }
}
