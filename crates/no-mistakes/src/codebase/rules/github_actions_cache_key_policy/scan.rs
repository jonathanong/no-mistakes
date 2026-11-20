use super::text::is_cache_action;
use super::walk::walk_jobs;
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

pub(super) fn analyze(source: &str, action: bool) -> Document {
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
    let jobs = walk_jobs(source, action);
    let seen = jobs
        .iter()
        .flat_map(|job| &job.steps)
        .filter(|step| step.uses.as_deref().is_some_and(is_cache_action))
        .count();
    if seen == cache_count(&value, action) {
        Document::Jobs(jobs)
    } else {
        Document::Unaligned
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
