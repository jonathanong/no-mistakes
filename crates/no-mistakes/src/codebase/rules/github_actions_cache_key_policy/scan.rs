use super::super::github_actions_workflow_files::step_lines;
use super::text::{is_cache_action, path_entries};
use serde_yaml::Value;

#[derive(Debug)]
pub(super) struct Field {
    pub line: usize,
    pub value: String,
}

#[derive(Debug)]
pub(super) struct Step {
    pub line: usize,
    pub id: Option<String>,
    pub uses: Option<String>,
    pub run: Option<String>,
    pub path: Option<Field>,
    pub key: Option<Field>,
    pub restore_keys: Option<Field>,
}

impl Step {
    pub(super) fn blank(line: usize) -> Self {
        Self {
            line,
            id: None,
            uses: None,
            run: None,
            path: None,
            key: None,
            restore_keys: None,
        }
    }
}

#[derive(Debug)]
pub(super) struct Job {
    pub steps: Vec<Step>,
}

#[derive(Debug)]
pub(super) enum Document {
    Jobs(Vec<Job>),
    Invalid { line: usize, detail: String },
    Unaligned,
    Ignored,
}

pub(super) fn analyze(source: &str, action: bool, configured: &[String]) -> Document {
    let source = source.trim_start_matches('\u{feff}');
    let value = match serde_yaml::from_str::<Value>(source) {
        Ok(value) => value,
        Err(error) => {
            return Document::Invalid {
                line: error.location().map_or(1, |loc| loc.line()),
                detail: error.to_string(),
            };
        }
    };
    if action && value["runs"]["using"].as_str() != Some("composite") {
        return Document::Ignored;
    }
    let jobs = map_jobs(step_lines::walk_steps(source, action));
    let seen = jobs
        .iter()
        .flat_map(|job| &job.steps)
        .filter(|step| step.uses.as_deref().is_some_and(is_cache_action))
        .count();
    if seen == cache_count(&value, action) || !configured_path(&value, &jobs, action, configured) {
        Document::Jobs(jobs)
    } else {
        Document::Unaligned
    }
}

fn map_jobs(jobs: Vec<step_lines::Job>) -> Vec<Job> {
    jobs.into_iter()
        .map(|job| Job {
            steps: job.steps.into_iter().map(map_step).collect(),
        })
        .collect()
}

fn map_step(step: step_lines::Step) -> Step {
    let mut out = Step::blank(step.line);
    for field in step.fields {
        let line = field.value_line;
        match field.name.as_str() {
            "id" => out.id = Some(field.value),
            "uses" => out.uses = Some(field.value),
            "run" => out.run = Some(field.value),
            "path" => {
                out.path = Some(Field {
                    line,
                    value: field.value,
                })
            }
            "key" => {
                out.key = Some(Field {
                    line,
                    value: field.value,
                })
            }
            "restore-keys" => {
                out.restore_keys = Some(Field {
                    line,
                    value: field.value,
                })
            }
            _ => {}
        }
    }
    out
}

fn configured_path(value: &Value, jobs: &[Job], action: bool, configured: &[String]) -> bool {
    if configured.is_empty() {
        return false;
    }
    serde_paths(value, action)
        .iter()
        .any(|path| configured.iter().any(|item| item == path))
        || jobs.iter().flat_map(|job| &job.steps).any(|step| {
            step.uses.as_deref().is_some_and(is_cache_action)
                && step.path.as_ref().is_some_and(|field| {
                    path_entries(&field.value)
                        .iter()
                        .any(|path| configured.iter().any(|item| item == path))
                })
        })
}

fn serde_paths(value: &Value, action: bool) -> Vec<String> {
    let mut paths = Vec::new();
    if action {
        push_paths(&mut paths, &value["runs"]["steps"]);
        return paths;
    }
    if let Some(jobs) = value["jobs"].as_mapping() {
        for job in jobs.values() {
            push_paths(&mut paths, &job["steps"]);
        }
    }
    paths
}

fn push_paths(paths: &mut Vec<String>, steps: &Value) {
    let Some(steps) = steps.as_sequence() else {
        return;
    };
    for step in steps {
        if !step
            .get("uses")
            .and_then(Value::as_str)
            .is_some_and(is_cache_action)
        {
            continue;
        }
        let Some(path) = step.get("with").and_then(|with| with.get("path")) else {
            continue;
        };
        match path {
            Value::String(text) => paths.extend(path_entries(text)),
            Value::Sequence(items) => {
                for item in items {
                    if let Some(text) = item.as_str() {
                        paths.extend(path_entries(text));
                    }
                }
            }
            _ => {}
        }
    }
}

fn cache_count(value: &Value, action: bool) -> usize {
    if action {
        return step_count(&value["runs"]["steps"]);
    }
    value["jobs"]
        .as_mapping()
        .map(|jobs| jobs.values().map(|job| step_count(&job["steps"])).sum())
        .unwrap_or(0)
}

fn step_count(steps: &Value) -> usize {
    steps
        .as_sequence()
        .map(|steps| {
            steps
                .iter()
                .filter(|step| {
                    step.get("uses")
                        .and_then(Value::as_str)
                        .is_some_and(is_cache_action)
                })
                .count()
        })
        .unwrap_or(0)
}
