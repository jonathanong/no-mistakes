use std::collections::BTreeSet;

pub(super) fn extract_yaml_sequence(source: &str, key: &str) -> BTreeSet<String> {
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(source) else {
        return BTreeSet::new();
    };
    let Some(serde_yaml::Value::Sequence(items)) = value_at_key(&value, key) else {
        return BTreeSet::new();
    };
    items
        .iter()
        .filter_map(|item| match item {
            serde_yaml::Value::String(value) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

/// Extracts terminal string values selected from a YAML document.
///
/// Selector segments are dot-separated. Bare mapping keys and numeric sequence
/// indexes select one value; `[]` visits every member of a sequence. Bracketed
/// JSON string segments select literal mapping keys that would otherwise be
/// structural, such as `["a.b"]`, `["[]"]`, or `["0"]`. Invalid selectors and
/// non-string terminal values are deliberately ignored, so callers can use
/// `minSize` to make required inventories fail closed.
pub(super) fn extract_yaml_string_selector(source: &str, selector: &str) -> BTreeSet<String> {
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(source) else {
        return BTreeSet::new();
    };
    let Some(segments) = parse_selector(selector) else {
        return BTreeSet::new();
    };
    let mut selected = Vec::new();
    select_values(&value, &segments, &mut selected);
    selected
        .into_iter()
        .filter_map(|value| match value {
            serde_yaml::Value::String(value) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

enum SelectorSegment {
    Key(String),
    Index(usize),
    Each,
}

fn parse_selector(selector: &str) -> Option<Vec<SelectorSegment>> {
    if selector.is_empty() {
        return None;
    }
    let bytes = selector.as_bytes();
    let mut index = 0;
    let mut segments = Vec::new();
    loop {
        let (segment, next) = if selector[index..].starts_with("[]")
            && matches!(bytes.get(index + 2), None | Some(b'.'))
        {
            (SelectorSegment::Each, index + 2)
        } else if bytes.get(index) == Some(&b'[') {
            parse_quoted_key_segment(selector, index)?
        } else {
            let next = selector[index..]
                .find('.')
                .map_or(selector.len(), |offset| index + offset);
            let key = &selector[index..next];
            if key.is_empty() {
                return None;
            }
            let segment = key
                .parse::<usize>()
                .map(SelectorSegment::Index)
                .unwrap_or_else(|_| SelectorSegment::Key(key.to_string()));
            (segment, next)
        };
        segments.push(segment);
        if next == selector.len() {
            return Some(segments);
        }
        if bytes.get(next) != Some(&b'.') || next + 1 == selector.len() {
            return None;
        }
        index = next + 1;
    }
}

fn parse_quoted_key_segment(selector: &str, index: usize) -> Option<(SelectorSegment, usize)> {
    if selector.as_bytes().get(index + 1) != Some(&b'"') {
        return None;
    }
    let quote_start = index + 1;
    let mut quote_end = quote_start + 1;
    let mut escaped = false;
    while quote_end < selector.len() {
        match selector.as_bytes()[quote_end] {
            b'\\' if !escaped => escaped = true,
            b'"' if !escaped => break,
            _ => escaped = false,
        }
        quote_end += 1;
    }
    if selector.as_bytes().get(quote_end) != Some(&b'"')
        || selector.as_bytes().get(quote_end + 1) != Some(&b']')
    {
        return None;
    }
    let key = serde_json::from_str(&selector[quote_start..=quote_end]).ok()?;
    Some((SelectorSegment::Key(key), quote_end + 2))
}

fn select_values<'a>(
    value: &'a serde_yaml::Value,
    segments: &[SelectorSegment],
    selected: &mut Vec<&'a serde_yaml::Value>,
) {
    let Some((segment, remaining)) = segments.split_first() else {
        selected.push(value);
        return;
    };
    let next = match segment {
        SelectorSegment::Each => {
            if let serde_yaml::Value::Sequence(items) = value {
                for item in items {
                    select_values(item, remaining, selected);
                }
            }
            return;
        }
        SelectorSegment::Index(index) => match value {
            serde_yaml::Value::Sequence(items) => items.get(*index),
            _ => None,
        },
        SelectorSegment::Key(key) => value.get(key),
    };
    if let Some(next) = next {
        select_values(next, remaining, selected);
    }
}

fn value_at_key<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a serde_yaml::Value> {
    key.split('.').try_fold(value, |current, part| {
        if let Ok(index) = part.parse::<usize>() {
            match current {
                serde_yaml::Value::Sequence(items) => items.get(index),
                _ => None,
            }
        } else {
            current.get(part)
        }
    })
}
