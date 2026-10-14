use std::collections::BTreeMap;

pub(super) struct Locations {
    lines: BTreeMap<String, usize>,
    declaration: usize,
}

impl Locations {
    pub(super) fn line(&self, job: &str) -> usize {
        self.lines.get(job).copied().unwrap_or(self.declaration)
    }
}

// Parse only direct job-key lines once, preserving YAML quoting and escapes.
pub(super) fn collect(source: &str) -> Locations {
    let mut locations = Locations {
        lines: BTreeMap::new(),
        declaration: 1,
    };
    let mut root_indent = None;
    let mut in_jobs = false;
    let mut job_indent = None;
    let mut pending_key = None;
    for (index, line) in source.lines().enumerate() {
        let text = line.trim();
        if text.is_empty()
            || text.starts_with(['#', '%'])
            || matches!(text.split_whitespace().next(), Some("---" | "..."))
        {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let root_indent = *root_indent.get_or_insert(indent);
        if in_jobs && indent == root_indent {
            break;
        }
        if indent != root_indent && (!in_jobs || indent != *job_indent.get_or_insert(indent)) {
            continue;
        }
        if let Some(key) = text.strip_prefix("? ") {
            pending_key = Some((decode_key(key), index + 1));
            continue;
        }
        let (key, key_line) = if text
            .strip_prefix(':')
            .is_some_and(|value| value.chars().next().is_none_or(char::is_whitespace))
        {
            // The document has already parsed, so explicit key/value pairing is valid.
            pending_key.take().unwrap_or((None, index + 1))
        } else {
            pending_key = None;
            (mapping_key(text), index + 1)
        };
        if indent == root_indent {
            in_jobs = key.as_deref() == Some("jobs");
            if in_jobs {
                locations.declaration = key_line;
            }
            continue;
        }
        if let Some(key) = key {
            locations.lines.insert(key, key_line);
        }
    }
    locations
}

fn mapping_key(text: &str) -> Option<String> {
    let mut quote = None;
    let mut escaped = false;
    let colon = text.char_indices().find_map(|(index, ch)| {
        if escaped {
            escaped = false;
        } else if quote == Some('"') && ch == '\\' {
            escaped = true;
        } else if Some(ch) == quote {
            quote = None;
        } else if quote.is_none() && matches!(ch, '\'' | '"') {
            quote = Some(ch);
        } else if quote.is_none()
            && ch == ':'
            && text[index + 1..]
                .chars()
                .next()
                .is_none_or(char::is_whitespace)
        {
            return Some(index);
        }
        None
    })?;
    // Ignore the value: a job alias can reference an anchor outside this line.
    decode_key(&text[..colon])
}

fn decode_key(text: &str) -> Option<String> {
    serde_yaml::from_str::<serde_yaml::Value>(text)
        .ok()
        .and_then(|key| key.as_str().map(str::to_owned))
}
