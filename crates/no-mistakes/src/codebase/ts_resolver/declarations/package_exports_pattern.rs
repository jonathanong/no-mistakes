//! One `*` in a `package.json` `exports` key, substituted into each target.
use super::Target;
use serde_json::{Map, Value};

pub(super) struct Selection<'a> {
    pub(super) value: &'a Value,
    pub(super) capture: Option<String>,
}

pub(super) fn exact<'a>(value: &'a Value) -> Selection<'a> {
    Selection {
        value,
        capture: None,
    }
}

/// Longest prefix wins, then the pattern name. A target without one `*` is skipped.
pub(super) fn pattern_export<'a>(map: &'a Map<String, Value>, key: &str) -> Option<Selection<'a>> {
    let mut best: Option<(&str, &Value, String)> = None;
    for (pattern, value) in map {
        let Some((capture, star)) = matched(pattern, key) else {
            continue;
        };
        if !value_stars(value) {
            continue;
        }
        let replace = best.as_ref().is_none_or(|(prior, _, _)| {
            star > star_at(prior) || (star == star_at(prior) && pattern.as_str() < *prior)
        });
        if replace {
            best = Some((pattern.as_str(), value, capture));
        }
    }
    best.map(|(_, value, capture)| Selection {
        value,
        capture: Some(capture),
    })
}

pub(super) fn apply_capture(targets: &mut [Target], capture: Option<&str>) -> Result<(), ()> {
    let Some(capture) = capture else {
        return Ok(());
    };
    for target in targets {
        if target.path.matches('*').count() != 1 {
            return Err(());
        }
        target.path = target.path.replacen('*', capture, 1);
    }
    Ok(())
}

fn matched(pattern: &str, key: &str) -> Option<(String, usize)> {
    let star = pattern.find('*')?;
    if pattern[star + 1..].contains('*') {
        return None;
    }
    let capture = key
        .strip_prefix(&pattern[..star])?
        .strip_suffix(&pattern[star + 1..])?;
    capture_ok(capture).then(|| (capture.to_string(), star))
}

fn star_at(pattern: &str) -> usize {
    pattern.find('*').unwrap_or(0)
}

fn capture_ok(capture: &str) -> bool {
    !capture.is_empty()
        && !capture.starts_with(['/', '\\'])
        && !capture.split(['/', '\\']).any(|part| part == "..")
}

fn value_stars(value: &Value) -> bool {
    let mut saw = false;
    stars(value, &mut saw) && saw
}

fn stars(value: &Value, saw: &mut bool) -> bool {
    match value {
        Value::String(text) => {
            *saw = true;
            text.matches('*').count() == 1
        }
        Value::Object(map) => map.values().all(|item| stars(item, saw)),
        Value::Array(_) => false,
        _ => true,
    }
}
