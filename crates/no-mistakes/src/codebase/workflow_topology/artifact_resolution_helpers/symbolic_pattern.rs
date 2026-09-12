//! Dynamic-template-vs-glob-pattern matching, split out of [`super`] to
//! stay under the crate's per-file line limit. Re-exported by [`super`] so
//! `artifact_resolution_helpers::symbolic_pattern_match` keeps working
//! unchanged for every external caller.

/// Whether `pattern` could match every possible expansion of a dynamic
/// `template` (an upload name containing `${{ ... }}` expressions we can't
/// evaluate): each expression must line up with a whole `*`/`**` wildcard
/// in `pattern`, with identical, glob-metacharacter-free literal text
/// around it. This never proves a match, only that one is *possible* — the
/// caller always records it with `match: "possible"`.
pub fn symbolic_pattern_match(template: &str, pattern: &str) -> bool {
    let template_chars: Vec<char> = template.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let expressions = find_expression_spans(&template_chars);
    if expressions.is_empty() {
        return false;
    }
    let mut template_cursor = 0usize;
    let mut pattern_cursor = 0usize;
    for (start, end) in &expressions {
        let literal = &template_chars[template_cursor..*start];
        if has_glob_syntax(literal) || !starts_with_at(&pattern_chars, pattern_cursor, literal) {
            return false;
        }
        pattern_cursor += literal.len();
        let Some(wildcard_len) = wildcard_prefix_len(&pattern_chars[pattern_cursor..]) else {
            return false;
        };
        pattern_cursor += wildcard_len;
        template_cursor = *end;
    }
    let suffix = &template_chars[template_cursor..];
    !has_glob_syntax(suffix) && pattern_chars[pattern_cursor..] == *suffix
}

fn has_glob_syntax(value: &[char]) -> bool {
    value.iter().any(|&c| {
        matches!(
            c,
            '*' | '?' | '[' | ']' | '{' | '}' | '(' | ')' | '!' | '+' | '@' | '\\'
        )
    })
}

fn starts_with_at(haystack: &[char], start: usize, needle: &[char]) -> bool {
    haystack.len() >= start + needle.len() && haystack[start..start + needle.len()] == *needle
}

fn wildcard_prefix_len(chars: &[char]) -> Option<usize> {
    if chars.starts_with(&['*', '*']) {
        Some(2)
    } else if chars.first() == Some(&'*') {
        Some(1)
    } else {
        None
    }
}

/// Finds every `${{ ... }}` span: a `${{` opener, one or more non-`}`
/// characters, then a `}}` closer — matching `/\$\{\{[^}]+\}\}/gu`.
fn find_expression_spans(chars: &[char]) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut i = 0usize;
    while i + 3 <= chars.len() {
        if chars[i] == '$' && chars[i + 1] == '{' && chars[i + 2] == '{' {
            let body_start = i + 3;
            let mut j = body_start;
            while j < chars.len() && chars[j] != '}' {
                j += 1;
            }
            if j > body_start && j + 1 < chars.len() && chars[j] == '}' && chars[j + 1] == '}' {
                spans.push((i, j + 2));
                i = j + 2;
                continue;
            }
        }
        i += 1;
    }
    spans
}
