use serde_yaml::{Mapping, Value};

/// Copy YAML 1.1 merge keys into their parent mapping, in place.
///
/// `serde_yaml` resolves aliases and then leaves `<<` as an ordinary key. An
/// explicit key already on the parent wins. In a sequence, an earlier mapping
/// wins over a later one. A source's own `<<` is applied before later sources,
/// so a merge inside a merged mapping is expanded too. A merge value that is
/// not a mapping, or a sequence of mappings, cannot be applied and is returned
/// as an error instead of being dropped.
pub(crate) fn expand_merge_keys(value: &mut Value) -> Result<(), String> {
    expand_located(value, &|_| Some(0)).map_err(|(_, message)| message)
}

pub(super) fn expand_located(
    value: &mut Value,
    line_for_key: &impl Fn(&Value) -> Option<usize>,
) -> Result<(), (usize, String)> {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            Value::Mapping(mapping) => {
                if let Some((merge, line)) = take_merge(mapping, line_for_key) {
                    merge_sources(mapping, merge, line, line_for_key)?;
                }
                let start = pending.len();
                pending.extend(mapping.values_mut());
                pending[start..].reverse();
            }
            Value::Sequence(items) => pending.extend(items.iter_mut().rev()),
            Value::Tagged(tagged) => pending.push(&mut tagged.value),
            _ => {}
        }
    }
    Ok(())
}

fn take_merge(
    mapping: &mut Mapping,
    line_for_key: &impl Fn(&Value) -> Option<usize>,
) -> Option<(Value, usize)> {
    if !mapping.contains_key("<<") {
        return None;
    }
    let key = mapping.keys().find(|key| key.as_str() == Some("<<"))?;
    let line = line_for_key(key)?;
    mapping.shift_remove("<<").map(|merge| (merge, line))
}

fn merge_sources(
    mapping: &mut Mapping,
    merge: Value,
    line: usize,
    line_for_key: &impl Fn(&Value) -> Option<usize>,
) -> Result<(), (usize, String)> {
    let mut pending = Vec::new();
    push_merge_sources(merge, &mut pending).map_err(|message| (line, message))?;
    // Each source's explicit keys win over its own merge, which is consumed
    // before later sequence sources. No merge source is copied or revisited.
    while let Some(mut source) = pending.pop() {
        if let Some((merge, line)) = take_merge(&mut source, line_for_key) {
            push_merge_sources(merge, &mut pending).map_err(|message| (line, message))?;
        }
        for (key, value) in source {
            mapping.entry(key).or_insert(value);
        }
    }
    Ok(())
}

fn push_merge_sources(merge: Value, pending: &mut Vec<Mapping>) -> Result<(), String> {
    match merge {
        Value::Mapping(source) => pending.push(source),
        Value::Sequence(sources) => {
            let start = pending.len();
            for source in sources {
                match source {
                    Value::Mapping(source) => pending.push(source),
                    other => return Err(reject(&other, "a mapping")),
                }
            }
            pending[start..].reverse();
        }
        other => return Err(reject(&other, "a mapping or list of mappings")),
    }
    Ok(())
}

fn reject(found: &Value, wanted: &str) -> String {
    match found {
        Value::Tagged(_) => "unexpected tagged value in merge".to_string(),
        Value::Sequence(_) => format!("expected {wanted} for merging, but found sequence"),
        _ => format!("expected {wanted} for merging, but found scalar"),
    }
}
