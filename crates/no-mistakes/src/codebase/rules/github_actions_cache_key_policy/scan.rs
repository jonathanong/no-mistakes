use super::super::github_actions_workflow_files::step_lines;
use super::text::{is_cache_action, path_entries};
use serde_yaml::Value;

#[derive(Debug)]
pub(super) struct Field {
    pub line: usize,
    pub value: String,
}

#[derive(Debug, Default)]
pub(super) struct Step {
    pub line: usize,
    pub id: Option<String>,
    pub uses: Option<String>,
    pub run: Option<String>,
    pub path: Option<Field>,
    pub key: Option<Field>,
    pub restore_keys: Option<Field>,
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
    let (serde_hits, walked_hits) = configured_hits(&value, &jobs, action, configured);
    let aligned = seen == cache_count(&value, action) && serde_hits == walked_hits;
    if aligned || (serde_hits.is_empty() && walked_hits.is_empty()) {
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
    let mut out = Step {
        line: step.line,
        ..Step::default()
    };
    for field in step.fields {
        match field.name.as_str() {
            "id" => out.id = Some(field.value),
            "uses" => out.uses = Some(field.value),
            "run" => out.run = Some(field.value),
            "path" => out.path = make_field(field),
            "key" => out.key = make_field(field),
            "restore-keys" => out.restore_keys = make_field(field),
            _ => {}
        }
    }
    out
}

fn make_field(field: step_lines::Field) -> Option<Field> {
    Some(Field {
        line: field.value_line,
        value: field.value,
    })
}

fn configured_hits(
    value: &Value,
    jobs: &[Job],
    action: bool,
    configured: &[String],
) -> (Vec<String>, Vec<String>) {
    if configured.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let keep = |paths: Vec<String>| {
        let mut hits: Vec<String> = paths
            .into_iter()
            .filter(|path| configured.iter().any(|item| item == path))
            .collect();
        hits.sort();
        hits
    };
    let walked = jobs
        .iter()
        .flat_map(|job| &job.steps)
        .filter(|step| step.uses.as_deref().is_some_and(is_cache_action))
        .filter_map(|step| step.path.as_ref())
        .flat_map(|field| path_entries(&field.value))
        .collect();
    (keep(serde_paths(value, action)), keep(walked))
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
