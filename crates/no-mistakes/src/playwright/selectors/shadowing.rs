use super::code_only_text::code_only_text;
use oxc_span::Span;
use regex::Regex;

pub(super) fn identifier_may_be_shadowed_or_reassigned(
    name: &str,
    span: Span,
    scope: Span,
    source: &str,
) -> bool {
    let start = scope.start as usize;
    let end = span.start as usize;
    let prefix = source.get(start..end).unwrap_or("");
    let prefix = code_only_text(prefix);
    let escaped = regex::escape(name);
    let declaration = Regex::new(&format!(r"\b(?:const|let|var)\s+{escaped}\b"))
        .expect("identifier declaration regex should compile");
    let destructuring_declaration = Regex::new(&format!(
        r"\b(?:const|let|var)\s+(?:\{{[^;]*\b{escaped}\b[^;]*\}}|\[[^;]*\b{escaped}\b[^;]*\])"
    ))
    .expect("identifier destructuring declaration regex should compile");
    let destructuring_parameter = Regex::new(&format!(
        r"\bfunction\b[^(]*\([^)]*(?:\{{[^)]*\b{escaped}\b[^)]*\}}|\[[^)]*\b{escaped}\b[^)]*\])"
    ))
    .expect("identifier destructuring parameter regex should compile");
    has_identifier_reassignment(&prefix, name)
        || declaration.is_match(&prefix)
        || destructuring_declaration.is_match(&prefix)
        || has_enclosing_shadow_binding(&prefix, &destructuring_parameter)
}

pub(super) fn has_identifier_reassignment(source: &str, name: &str) -> bool {
    let source = code_only_text(source);
    for (index, _) in source.match_indices(name) {
        let before = source[..index].chars().next_back();
        let after_index = index + name.len();
        let after = source[after_index..].chars().next();
        if before.is_some_and(is_identifier_continue) || after.is_some_and(is_identifier_continue) {
            continue;
        }
        let before = source[..index].trim_end();
        if before.ends_with("++") || before.ends_with("--") {
            return true;
        }
        let rest = source[after_index..].trim_start();
        if rest.starts_with("++") || rest.starts_with("--") {
            return true;
        }
        if [
            "+=", "-=", "*=", "/=", "%=", "**=", "&&=", "||=", "??=", "<<=", ">>=", ">>>=",
        ]
        .iter()
        .any(|operator| rest.starts_with(operator))
        {
            return true;
        }
        if let Some(after_equals) = rest.strip_prefix('=') {
            if !after_equals.starts_with('=')
                && !after_equals.starts_with('>')
                && !looks_like_jsx_attribute_expression(before, after_equals)
            {
                return true;
            }
        }
    }
    false
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()
}

fn looks_like_jsx_attribute_expression(before: &str, after_equals: &str) -> bool {
    if !after_equals.starts_with('{') || !has_unclosed_jsx_start(before) {
        return false;
    }
    let Some(close_brace) = matching_close_brace(after_equals) else {
        return true;
    };
    let rest = after_equals[close_brace + 1..].trim_start();
    rest.is_empty()
        || rest.starts_with('>')
        || rest.starts_with("/>")
        || rest.chars().next().is_some_and(is_jsx_attribute_start)
}

fn has_unclosed_jsx_start(before: &str) -> bool {
    let Some(open_index) = before.rfind('<') else {
        return false;
    };
    let tail = &before[open_index + 1..];
    let Some(first) = tail.chars().next() else {
        return false;
    };
    if !(first == '/' || first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    !tail.contains('>') && !tail.contains(';')
}

fn matching_close_brace(source: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in source.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn is_jsx_attribute_start(ch: char) -> bool {
    ch == '_' || ch == ':' || ch.is_ascii_alphabetic()
}

pub(super) fn has_enclosing_shadow_binding(prefix: &str, binding: &Regex) -> bool {
    binding.find_iter(prefix).any(|matched| {
        let rest = &prefix[matched.end()..];
        let Some(block_start) = rest.find('{') else {
            return false;
        };
        if rest[..block_start].contains(';') {
            return false;
        }
        let mut depth = 0usize;
        for ch in rest[block_start..].chars() {
            match ch {
                '{' => depth += 1,
                '}' if depth <= 1 => return false,
                '}' => depth -= 1,
                _ => {}
            }
        }
        depth > 0
    })
}

#[cfg(test)]
mod tests;
