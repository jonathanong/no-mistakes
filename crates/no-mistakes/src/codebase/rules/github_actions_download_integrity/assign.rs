/// Expands one whole-line uppercase literal. Any other uppercase assignment
/// in the same script leaves every `$NAME` untouched.
pub(super) fn expand_uppercase(script: &str) -> String {
    probe::note();
    let env = single_literal_binding(script);
    let mut out = String::new();
    for line in script.lines() {
        if matches!(classify(line.trim()), Kind::Literal(_, _)) && env.is_some() {
            out.push_str(line);
        } else if let Some((name, value)) = &env {
            out.push_str(&substitute(line, name, value));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// The same one-binding proof used by max, without expanding any script values.
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
    substitute(token, &binding.0, &binding.1)
}

#[cfg(test)]
#[path = "assign/probe_tests.rs"]
pub(super) mod probe;
#[cfg(not(test))]
#[path = "assign/probe_disabled.rs"]
mod probe;

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

fn substitute(line: &str, name: &str, value: &str) -> String {
    let mut out = String::new();
    let mut chars = line.chars().peekable();
    let mut quote = None;
    while let Some(ch) = chars.next() {
        if quote == Some('\'') {
            out.push(ch);
            if ch == '\'' {
                quote = None;
            }
            continue;
        }
        if ch == '\'' || ch == '"' {
            quote = if quote == Some(ch) { None } else { Some(ch) };
            out.push(ch);
            continue;
        }
        if ch == '$' {
            let found = read_expansion(&mut chars);
            if found == name {
                out.push_str(value);
                continue;
            }
            out.push('$');
            if !found.is_empty() {
                out.push_str(&found);
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn read_expansion(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    if chars.peek() == Some(&'{') {
        chars.next();
        let mut name = String::new();
        for next in chars.by_ref() {
            if next == '}' {
                break;
            }
            name.push(next);
        }
        return name;
    }
    let mut name = String::new();
    while chars
        .peek()
        .is_some_and(|next| next.is_ascii_uppercase() || next.is_ascii_digit() || *next == '_')
    {
        name.push(chars.next().unwrap_or('_'));
    }
    name
}
