use super::super::globs::{compile_patterns, CompiledGlob};
use serde_yaml::Value;

#[derive(Clone, Debug)]
pub(super) enum WorkflowPathFilters {
    Unrestricted,
    Restricted(Vec<WorkflowPathPredicate>),
    NotFileTriggered,
}

#[derive(Clone, Debug)]
pub(super) enum WorkflowPathPredicate {
    Paths(Vec<CompiledGlob>),
    PathsIgnore(Vec<CompiledGlob>),
}

impl WorkflowPathFilters {
    pub(super) fn allows(&self, path: &str) -> bool {
        match self {
            WorkflowPathFilters::Unrestricted => true,
            WorkflowPathFilters::Restricted(predicates) => {
                predicates.iter().any(|predicate| predicate.allows(path))
            }
            WorkflowPathFilters::NotFileTriggered => false,
        }
    }
}

impl WorkflowPathPredicate {
    fn allows(&self, path: &str) -> bool {
        match self {
            WorkflowPathPredicate::Paths(patterns) => super::selected_by(patterns, path),
            WorkflowPathPredicate::PathsIgnore(patterns) => !super::selected_by(patterns, path),
        }
    }
}

pub(super) fn workflow_path_filters(value: &Value) -> WorkflowPathFilters {
    let mut predicates = Vec::new();
    let mut has_file_event = false;
    let Some(on) = value.get("on") else {
        return WorkflowPathFilters::NotFileTriggered;
    };
    for (name, event) in workflow_events(on) {
        let Some(kind) = FileEventKind::from_name(name) else {
            continue;
        };
        let Some(event) = file_event(kind, event) else {
            continue;
        };
        has_file_event = true;
        match event {
            EventPaths::Unrestricted => return WorkflowPathFilters::Unrestricted,
            EventPaths::Paths(paths) => predicates.push(WorkflowPathPredicate::Paths(paths)),
            EventPaths::PathsIgnore(paths) => {
                predicates.push(WorkflowPathPredicate::PathsIgnore(paths));
            }
        }
    }
    if has_file_event {
        WorkflowPathFilters::Restricted(predicates)
    } else {
        WorkflowPathFilters::NotFileTriggered
    }
}

#[derive(Clone, Copy)]
enum FileEventKind {
    PullRequest,
    Push,
}

impl FileEventKind {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "pull_request" | "pull_request_target" => Some(Self::PullRequest),
            "push" => Some(Self::Push),
            _ => None,
        }
    }
}

enum EventPaths {
    Unrestricted,
    Paths(Vec<CompiledGlob>),
    PathsIgnore(Vec<CompiledGlob>),
}

fn workflow_events(on: &Value) -> Vec<(&str, Option<&Value>)> {
    match on {
        Value::Mapping(events) => events
            .iter()
            .filter_map(|(name, event)| name.as_str().map(|name| (name, Some(event))))
            .collect(),
        Value::Sequence(events) => events
            .iter()
            .filter_map(|event| event.as_str().map(|name| (name, None)))
            .collect(),
        Value::String(name) => vec![(name.as_str(), None)],
        _ => Vec::new(),
    }
}

fn file_event(kind: FileEventKind, event: Option<&Value>) -> Option<EventPaths> {
    let Some(map) = event.and_then(Value::as_mapping) else {
        return Some(EventPaths::Unrestricted);
    };
    if matches!(kind, FileEventKind::Push) && push_is_tag_only(map) {
        return None;
    }
    if let Some(paths) = event_path_patterns(map, "paths") {
        return Some(EventPaths::Paths(paths));
    }
    if let Some(paths) = event_path_patterns(map, "paths-ignore") {
        return Some(EventPaths::PathsIgnore(paths));
    }
    Some(EventPaths::Unrestricted)
}

fn push_is_tag_only(map: &serde_yaml::Mapping) -> bool {
    (map_contains_key(map, "tags") || map_contains_key(map, "tags-ignore"))
        && !map_contains_key(map, "branches")
        && !map_contains_key(map, "branches-ignore")
}

fn event_path_patterns(map: &serde_yaml::Mapping, key: &str) -> Option<Vec<CompiledGlob>> {
    let patterns = map
        .get(Value::String(key.to_string()))
        .and_then(Value::as_sequence)?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    if patterns.is_empty() {
        None
    } else {
        compile_patterns(&patterns).ok()
    }
}

fn map_contains_key(map: &serde_yaml::Mapping, key: &str) -> bool {
    map.contains_key(Value::String(key.to_string()))
}
