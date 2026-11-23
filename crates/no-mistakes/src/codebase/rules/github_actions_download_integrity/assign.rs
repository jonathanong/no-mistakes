mod bindings;
pub(super) use bindings::Bindings;
mod rewrite;
pub(super) use rewrite::prepare_max_script;

/// Recognize a low-effort command alias without expanding any URL values.
pub(super) fn single_literal_binding(script: &str) -> Option<(String, String)> {
    probe::note_binding_scan();
    let mut count = 0usize;
    let mut literal = None;
    for line in script.lines() {
        match classify(line.trim()) {
            Kind::None => {}
            Kind::Literal(name, value) => {
                count += 1;
                literal = Some((name, value));
            }
            Kind::Other => count += 1,
        }
    }
    if count == 1 {
        literal
    } else {
        None
    }
}

pub(super) fn substitute_command(token: &str, binding: &(String, String)) -> String {
    let mut out = rewrite::Script::default();
    out.substitute(token, 0, &mut &[][..], |name| {
        (name == binding.0).then_some(binding.1.as_str())
    });
    out.text
}

#[cfg(test)]
#[path = "assign/probe_tests.rs"]
pub(super) mod probe;
#[cfg(not(test))]
#[path = "assign/probe_disabled.rs"]
mod probe;

fn reference_name(text: &str) -> &str {
    text.strip_prefix('{')
        .and_then(|text| text.strip_suffix('}'))
        .unwrap_or(text)
}

enum Kind {
    None,
    Literal(String, String),
    Other,
}

fn classify(trimmed: &str) -> Kind {
    let (name, raw) = match trimmed.split_once('=') {
        Some(pair) => pair,
        None => return Kind::None,
    };
    if !uppercase_name(name) {
        return Kind::None;
    }
    match literal_value(raw) {
        Some(value) => Kind::Literal(name.to_string(), value),
        None => Kind::Other,
    }
}

fn uppercase_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|ch| ch.is_ascii_uppercase())
        && chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

fn literal_value(raw: &str) -> Option<String> {
    let raw = raw.trim_end();
    if let Some(value) = quoted(raw) {
        return clean(value);
    }
    if raw.is_empty() || raw.contains(char::is_whitespace) {
        return None;
    }
    clean(raw.to_string())
}

fn clean(value: String) -> Option<String> {
    if value.contains(['$', '`', '(', ')', '{', '}', '\\']) {
        None
    } else {
        Some(value)
    }
}

fn quoted(raw: &str) -> Option<String> {
    let mut chars = raw.chars();
    let quote = chars.next().filter(|ch| *ch == '"' || *ch == '\'')?;
    let mut value = String::new();
    for ch in chars.by_ref() {
        if ch == quote {
            let rest: String = chars.collect();
            let rest = rest.trim();
            if rest.is_empty() || rest.starts_with('#') {
                return Some(value);
            }
            return None;
        }
        value.push(ch);
    }
    None
}
