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
    expand_merge_keys_indexed(value).map_err(|(_, error)| error)
}

/// Expand merge keys in document order and return the 0-based index of the
/// merge key that failed, alongside the error message.
pub(super) fn expand_merge_keys_indexed(value: &mut Value) -> Result<(), (usize, String)> {
    let mut pending = vec![value];
    let mut merge_index = 0usize;
    while let Some(value) = pending.pop() {
        match value {
            Value::Mapping(mapping) => {
                if let Some(merge) = mapping.shift_remove("<<") {
                    merge_sources(mapping, merge).map_err(|error| (merge_index, error))?;
                    merge_index += 1;
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

fn merge_sources(mapping: &mut Mapping, merge: Value) -> Result<(), String> {
    let mut pending = Vec::new();
    push_merge_sources(merge, &mut pending)?;
    // Each source's explicit keys win over its own merge, which is consumed
    // before later sequence sources. No merge source is copied or revisited.
    while let Some(mut source) = pending.pop() {
        if let Some(merge) = source.shift_remove("<<") {
            push_merge_sources(merge, &mut pending)?;
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

/// The line of the `merge_index`-th merge key in document `doc_index`.
pub(super) fn merge_line(source: &str, doc_index: usize, merge_index: usize) -> usize {
    let mut current_doc = 0usize;
    let mut doc_started = false;
    let mut doc_start_line = 1usize;
    let mut first_in_doc = None;
    let mut seen_in_doc = 0usize;
    for (index, line) in source.lines().enumerate() {
        let line_no = index + 1;
        if is_doc_start(line) {
            if doc_started {
                current_doc += 1;
                if current_doc == doc_index {
                    doc_start_line = line_no;
                }
            }
            doc_started = true;
            continue;
        }
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            doc_started = true;
        }
        if current_doc == doc_index && has_merge_key(line) {
            first_in_doc.get_or_insert(line_no);
            if seen_in_doc == merge_index {
                return line_no;
            }
            seen_in_doc += 1;
        }
    }
    first_in_doc.unwrap_or(doc_start_line)
}

fn is_doc_start(line: &str) -> bool {
    line.strip_prefix("---")
        .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', '\t']))
}

fn has_merge_key(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut quote = 0u8;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if quote != 0 {
            if quote == b'"' && byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == quote {
                quote = 0;
            }
            index += 1;
            continue;
        }
        if byte == b'#' && (index == 0 || bytes[index - 1].is_ascii_whitespace()) {
            break;
        }
        if matches!(byte, b'\'' | b'"') {
            quote = byte;
            index += 1;
            continue;
        }
        if bytes[index..].starts_with(b"<<:") {
            let before_ok = index == 0
                || bytes[index - 1].is_ascii_whitespace()
                || matches!(bytes[index - 1], b'-' | b'{' | b',');
            let after_ok = bytes
                .get(index + 3)
                .is_none_or(|next| next.is_ascii_whitespace());
            if before_ok && after_ok {
                return true;
            }
        }
        index += 1;
    }
    false
}
