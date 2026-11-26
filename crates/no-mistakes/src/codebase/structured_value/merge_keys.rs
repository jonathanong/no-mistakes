use super::partial_yaml::{mapping_id, UnknownMappings};
use serde_yaml::{Mapping, Value};

#[derive(Clone, Copy)]
pub(super) enum KeyLocation {
    Merge(usize),
    Literal,
    Unlocated,
}

pub(super) enum MergeError {
    MissingLocation(usize),
    Invalid { line: usize, message: String },
}

pub(super) type KeyLocationResult = Result<KeyLocation, MergeError>;

/// Copy YAML 1.1 merge keys into their parent mapping, in place.
///
/// `serde_yaml` resolves aliases and then leaves `<<` as an ordinary key. An
/// explicit key already on the parent wins. In a sequence, an earlier mapping
/// wins over a later one. A source's own `<<` is applied before later sources,
/// so a merge inside a merged mapping is expanded too. A merge value that is
/// not a mapping, or a sequence of mappings, cannot be applied and is returned
/// as an error instead of being dropped.
pub(crate) fn expand_merge_keys(value: &mut Value) -> Result<(), String> {
    expand_located(value, &|_| Ok(KeyLocation::Merge(0)))
        .map(|_| ())
        .map_err(|error| error.detail().1)
}

pub(super) fn expand_located(
    value: &mut Value,
    locate_key: &impl Fn(&str) -> KeyLocationResult,
) -> Result<UnknownMappings, MergeError> {
    let mut incomplete = UnknownMappings::default();
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            Value::Mapping(mapping) => {
                let unresolved = match take_merge(mapping, locate_key)? {
                    MergeEntry::Absent => false,
                    MergeEntry::Unlocated => true,
                    MergeEntry::Located(merge, line) => {
                        merge_sources(mapping, merge, line, locate_key)?
                    }
                };
                if unresolved {
                    incomplete.insert(mapping_id(mapping));
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
    Ok(incomplete)
}

enum MergeEntry {
    Absent,
    Located(Value, usize),
    Unlocated,
}

fn take_merge(
    mapping: &mut Mapping,
    locate_key: &impl Fn(&str) -> KeyLocationResult,
) -> Result<MergeEntry, MergeError> {
    if !mapping.contains_key("<<") {
        return Ok(MergeEntry::Absent);
    }
    let key = mapping
        .keys()
        .filter_map(Value::as_str)
        .find(|key| *key == "<<")
        .expect("mapping contains the string key");
    let location = locate_key(key)?;
    if matches!(location, KeyLocation::Literal) {
        return Ok(MergeEntry::Absent);
    }
    let merge = mapping
        .shift_remove("<<")
        .expect("mapping contains the string key");
    match location {
        KeyLocation::Merge(line) => Ok(MergeEntry::Located(merge, line)),
        _ => Ok(MergeEntry::Unlocated),
    }
}

fn merge_sources(
    mapping: &mut Mapping,
    merge: Value,
    line: usize,
    locate_key: &impl Fn(&str) -> KeyLocationResult,
) -> Result<bool, MergeError> {
    let mut pending = Vec::new();
    push_merge_sources(merge, &mut pending)
        .map_err(|message| MergeError::Invalid { line, message })?;
    // Each source's explicit keys win over its own merge, which is consumed
    // before later sequence sources. No merge source is copied or revisited.
    while let Some(mut source) = pending.pop() {
        let unresolved = match take_merge(&mut source, locate_key)? {
            MergeEntry::Absent => false,
            MergeEntry::Unlocated => true,
            MergeEntry::Located(merge, line) => {
                push_merge_sources(merge, &mut pending)
                    .map_err(|message| MergeError::Invalid { line, message })?;
                false
            }
        };
        for (key, value) in source {
            mapping.entry(key).or_insert(value);
        }
        // The unknown source may supply any of the later sources' keys.
        if unresolved {
            return Ok(true);
        }
    }
    Ok(false)
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

impl MergeError {
    pub(super) fn detail(self) -> (usize, String) {
        match self {
            Self::MissingLocation(line) => (
                line,
                "could not match a YAML `<<` key to its source location".to_string(),
            ),
            Self::Invalid { line, message } => (line, message),
        }
    }
}
