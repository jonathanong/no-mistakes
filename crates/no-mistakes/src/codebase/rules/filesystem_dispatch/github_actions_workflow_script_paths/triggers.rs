mod glob;

use glob::{
    compile_pattern_sequence, find_key_line_after, find_top_key_line, selected_by, CompiledGlob,
};
use serde_yaml::{Mapping, Value};

pub(super) struct TriggerAnalysis {
    pub(super) filters: Vec<EventFilter>,
    pub(super) unanalyzable: Vec<TriggerIssue>,
}

pub(super) struct EventFilter {
    pub(super) event: String,
    pub(super) kind: FilterKind,
    globs: Vec<CompiledGlob>,
}

#[derive(Clone, Copy)]
pub(super) enum FilterKind {
    Paths,
    PathsIgnore,
}

pub(super) struct TriggerIssue {
    pub(super) line: usize,
    pub(super) detail: String,
}

impl EventFilter {
    pub(super) fn covers(&self, path: &str) -> bool {
        let matched = selected_by(&self.globs, path);
        match self.kind {
            FilterKind::Paths => matched,
            FilterKind::PathsIgnore => !matched,
        }
    }

    pub(super) fn label(&self) -> String {
        let field = match self.kind {
            FilterKind::Paths => "paths",
            FilterKind::PathsIgnore => "paths-ignore",
        };
        format!("on.{}.{field}", self.event)
    }
}

pub(super) fn parse_triggers(source: &str, root: &Value, events: &[String]) -> TriggerAnalysis {
    let mut out = TriggerAnalysis {
        filters: Vec::new(),
        unanalyzable: Vec::new(),
    };
    let Some(map) = root.as_mapping() else {
        out.unanalyzable.push(TriggerIssue {
            line: 1,
            detail: "workflow root is not a YAML mapping; rewrite it as a standard workflow mapping so path filters can be checked.".to_string(),
        });
        return out;
    };
    let on_value = map.get(Value::String("on".into()));
    if on_value.is_none() {
        if has_merge_or_non_string_key(map) {
            out.unanalyzable.push(TriggerIssue {
                line: 1,
                detail: "workflow root uses YAML merge or non-string keys without a literal `on` key; write `on:` directly so path filters can be checked.".to_string(),
            });
        }
        return out;
    }
    let on_line = find_top_key_line(source, "on")
        .or_else(|| find_key_line_after(source, 1, "on"))
        .unwrap_or(1);
    match on_value.expect("checked above") {
        Value::String(_) => {}
        Value::Sequence(items) => {
            if items.iter().any(|item| !item.is_string()) {
                out.unanalyzable.push(TriggerIssue {
                    line: on_line,
                    detail: "`on` sequence contains non-string entries; list literal event names so path filters can be checked.".to_string(),
                });
            }
        }
        Value::Mapping(event_map) => parse_event_map(source, event_map, events, on_line, &mut out),
        _ => out.unanalyzable.push(TriggerIssue {
            line: on_line,
            detail: "`on` trigger uses an unsupported YAML shape; write a mapping, sequence, or event name so path filters can be checked.".to_string(),
        }),
    }
    out
}

fn parse_event_map(
    source: &str,
    event_map: &Mapping,
    events: &[String],
    on_line: usize,
    out: &mut TriggerAnalysis,
) {
    if has_merge_or_non_string_key(event_map) {
        out.unanalyzable.push(TriggerIssue {
            line: on_line,
            detail: "`on` mapping uses YAML merge or non-string keys; write each trigger event explicitly so path filters can be checked.".to_string(),
        });
    }
    for (key, value) in event_map {
        let Some(name) = key.as_str() else { continue };
        if !events.iter().any(|allowed| allowed == name) {
            continue;
        }
        let event_line = find_key_line_after(source, on_line, name).unwrap_or(on_line);
        match value {
            Value::Null => {}
            Value::Mapping(config) => parse_event_config(source, name, config, event_line, out),
            _ => out.unanalyzable.push(TriggerIssue {
                line: event_line,
                detail: format!("`on.{name}` uses a non-mapping trigger value; write a mapping or empty value so path filters can be checked."),
            }),
        }
    }
}

fn parse_event_config(
    source: &str,
    event: &str,
    config: &Mapping,
    event_line: usize,
    out: &mut TriggerAnalysis,
) {
    if has_merge_or_non_string_key(config) {
        out.unanalyzable.push(TriggerIssue {
            line: event_line,
            detail: format!("`on.{event}` uses YAML merge or non-string keys; write `paths` or `paths-ignore` directly."),
        });
        return;
    }
    let paths = config.get(Value::String("paths".into()));
    let ignore = config.get(Value::String("paths-ignore".into()));
    if paths.is_some() && ignore.is_some() {
        out.unanalyzable.push(TriggerIssue {
            line: event_line,
            detail: format!(
                "`on.{event}` sets both `paths` and `paths-ignore`, which GitHub Actions rejects."
            ),
        });
        return;
    }
    let Some((kind, field, raw)) = paths
        .map(|val| (FilterKind::Paths, "paths", val))
        .or_else(|| ignore.map(|val| (FilterKind::PathsIgnore, "paths-ignore", val)))
    else {
        return;
    };
    let line = find_key_line_after(source, event_line, field).unwrap_or(event_line);
    match compile_pattern_sequence(raw) {
        Ok(globs) => out.filters.push(EventFilter {
            event: event.to_string(),
            kind,
            globs,
        }),
        Err(reason) => out.unanalyzable.push(TriggerIssue {
            line,
            detail: format!("`on.{event}.{field}` {reason}"),
        }),
    }
}

fn has_merge_or_non_string_key(map: &Mapping) -> bool {
    map.keys()
        .any(|key| key.as_str().is_none_or(|name| name == "<<"))
}
