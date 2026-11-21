use super::entries;
use super::select;
use crate::codebase::structured_value::parse_yaml_documents;
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

pub(super) struct ParsedConfig {
    pub(super) relative: String,
    pub(super) project: String,
    pub(super) source: String,
    pub(super) documents: Vec<Value>,
}

pub(super) fn load(
    root: &Path,
    files: &[String],
    tracked: &[String],
    tracked_set: &BTreeSet<&str>,
) -> (Vec<ParsedConfig>, Vec<super::RuleFinding>) {
    let mut findings = Vec::new();
    let (seeds, missing) = select::seeds(files, tracked_set);
    for relative in missing {
        findings.push(super::finding(
            &relative,
            1,
            format!(
                "configured config `{relative}` is not tracked. Point files at a tracked ast-grep config, rule directory, or rule YAML file."
            ),
            Some(relative.clone()),
        ));
    }
    let mut seen = BTreeMap::new();
    let mut queue = VecDeque::new();
    let mut parsed = Vec::new();
    for relative in seeds {
        remember(&mut seen, &mut queue, &mut parsed, relative, String::new());
    }
    while let Some(relative) = queue.pop_front() {
        let inherited = seen
            .get(&relative)
            .map(|visit| visit.project.clone())
            .unwrap_or_default();
        let source = match std::fs::read_to_string(root.join(&relative)) {
            Ok(source) => source,
            Err(error) => {
                findings.push(super::finding(
                    &relative,
                    1,
                    format!("failed to read `{relative}` ({error})."),
                    None,
                ));
                continue;
            }
        };
        let documents = parse_yaml_documents(&source);
        for (line, message) in documents.errors {
            findings.push(super::finding(&relative, line, message, None));
        }
        if documents.values.is_empty() {
            continue;
        }
        let project = project_for(&relative, &documents.values, &inherited);
        let index = parsed.len();
        let visit = seen
            .get_mut(&relative)
            .expect("queued config was remembered");
        visit.parsed = Some(index);
        if visit.project.is_empty() && !project.is_empty() {
            visit.project = project.clone();
        }
        for value in &documents.values {
            enqueue_rule_dirs(
                &relative,
                value,
                tracked,
                &project,
                &mut seen,
                &mut queue,
                &mut parsed,
            );
        }
        parsed.push(ParsedConfig {
            relative,
            project,
            source,
            documents: documents.values,
        });
    }
    (parsed, findings)
}

fn project_for(relative: &str, documents: &[Value], inherited: &str) -> String {
    let owns_project = documents
        .iter()
        .any(|value| !entries::rule_directories(value).is_empty());
    if owns_project {
        config_directory(relative)
    } else {
        inherited.to_string()
    }
}

fn config_directory(relative: &str) -> String {
    match Path::new(relative).parent() {
        Some(parent) if !parent.as_os_str().is_empty() => {
            parent.to_string_lossy().replace('\\', "/")
        }
        _ => String::new(),
    }
}

struct Visit {
    project: String,
    parsed: Option<usize>,
}

fn enqueue_rule_dirs(
    relative: &str,
    value: &Value,
    tracked: &[String],
    project: &str,
    seen: &mut BTreeMap<String, Visit>,
    queue: &mut VecDeque<String>,
    parsed: &mut [ParsedConfig],
) {
    for dir in entries::rule_directories(value) {
        let Some(prefix) = select::child_prefix(relative, &dir) else {
            continue;
        };
        for candidate in tracked {
            if candidate.starts_with(&prefix) && select::is_yaml(candidate) {
                remember(seen, queue, parsed, candidate.clone(), project.to_string());
            }
        }
    }
}

fn remember(
    seen: &mut BTreeMap<String, Visit>,
    queue: &mut VecDeque<String>,
    parsed: &mut [ParsedConfig],
    relative: String,
    project: String,
) {
    if let Some(visit) = seen.get_mut(&relative) {
        if visit.project.is_empty() && !project.is_empty() {
            visit.project = project.clone();
            if let Some(index) = visit.parsed {
                parsed[index].project = project;
            }
        }
        return;
    }
    seen.insert(
        relative.clone(),
        Visit {
            project,
            parsed: None,
        },
    );
    queue.push_back(relative);
}

#[cfg(test)]
#[path = "scan_tests.rs"]
mod tests;
