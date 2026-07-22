//! Low-level `Vec<char>` scanning primitives shared by [`super`]'s
//! reference extractors, split out to stay under the crate's per-file line
//! limit. Not part of the module's public API — every item here is
//! `pub(super)`, called only from the extraction logic in [`super`].

/// A single `.name` or `['name']` / `["name"]` access starting at `start`.
/// Returns the extracted name and the absolute end index.
pub(super) fn static_access(chars: &[char], start: usize) -> Option<(String, usize)> {
    let tail = &chars[start..];
    if let Some((name, consumed)) = match_dot_access(tail) {
        return Some((name, start + consumed));
    }
    if let Some((name, consumed)) = match_bracket_access(tail) {
        return Some((name, start + consumed));
    }
    None
}

/// Matches `^\s*\.\s*([A-Za-z_][A-Za-z0-9_-]*)`.
fn match_dot_access(chars: &[char]) -> Option<(String, usize)> {
    let mut index = skip_whitespace(chars, 0);
    if chars.get(index) != Some(&'.') {
        return None;
    }
    index += 1;
    index = skip_whitespace(chars, index);
    let ident_start = index;
    match chars.get(index) {
        Some(c) if c.is_ascii_alphabetic() || *c == '_' => index += 1,
        _ => return None,
    }
    while matches!(chars.get(index), Some(c) if c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
    {
        index += 1;
    }
    Some((chars[ident_start..index].iter().collect(), index))
}

/// Matches `^\s*\[\s*(['"])([^'"]+)\1\s*\]`.
fn match_bracket_access(chars: &[char]) -> Option<(String, usize)> {
    let mut index = skip_whitespace(chars, 0);
    if chars.get(index) != Some(&'[') {
        return None;
    }
    index += 1;
    index = skip_whitespace(chars, index);
    let quote = *chars.get(index)?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    index += 1;
    let content_start = index;
    while matches!(chars.get(index), Some(c) if *c != '\'' && *c != '"') {
        index += 1;
    }
    if index == content_start || chars.get(index) != Some(&quote) {
        return None;
    }
    let content: String = chars[content_start..index].iter().collect();
    index += 1;
    index = skip_whitespace(chars, index);
    if chars.get(index) != Some(&']') {
        return None;
    }
    Some((content, index + 1))
}

fn skip_whitespace(chars: &[char], start: usize) -> usize {
    let mut index = start;
    while matches!(chars.get(index), Some(c) if c.is_whitespace()) {
        index += 1;
    }
    index
}

/// True at a string boundary or any character that can't be part of a
/// `[-.A-Za-z0-9_]` access chain — i.e. `needs`/`steps` isn't preceded by
/// something that would make it part of a larger identifier.
pub(super) fn is_access_boundary(character: Option<char>) -> bool {
    match character {
        None => true,
        Some(c) => !(c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_'),
    }
}

/// Walks backward from `start` (which may be `-1`, meaning "before the
/// string") skipping whitespace; returns the first non-whitespace
/// character found, or `None` if the start of the string is reached.
pub(super) fn previous_non_whitespace(chars: &[char], start: isize) -> Option<char> {
    let mut index = start;
    while index >= 0 && chars[index as usize].is_whitespace() {
        index -= 1;
    }
    if index < 0 {
        None
    } else {
        Some(chars[index as usize])
    }
}

/// `start` is the index of an opening quote; returns the index just past
/// the matching closing quote (doubled quotes `''`/`\"\"` are an escaped
/// literal quote, not a terminator), or `chars.len()` if unterminated.
pub(super) fn quoted_end(chars: &[char], start: usize, quote: char) -> usize {
    let mut index = start + 1;
    while index < chars.len() {
        if chars[index] != quote {
            index += 1;
            continue;
        }
        if chars.get(index + 1) == Some(&quote) {
            index += 2;
            continue;
        }
        return index + 1;
    }
    chars.len()
}

/// Finds every `${{ ... }}` span and returns each span's inner content as
/// its own char buffer, honoring quoted `}}` the same way [`quoted_end`]
/// does. An unterminated `${{` is dropped, matching the TS engine.
pub(super) fn embedded_expressions(chars: &[char]) -> Vec<Vec<char>> {
    const OPEN: [char; 3] = ['$', '{', '{'];
    let mut expressions = Vec::new();
    let mut index = 0usize;
    while let Some(start) = find_subsequence(chars, &OPEN, index) {
        let mut end = start + 3;
        while end < chars.len() {
            let character = chars[end];
            if character == '\'' || character == '"' {
                end = quoted_end(chars, end, character);
                continue;
            }
            if chars[end..].starts_with(&['}', '}']) {
                break;
            }
            end += 1;
        }
        if end >= chars.len() {
            break;
        }
        expressions.push(chars[start + 3..end].to_vec());
        index = end + 2;
    }
    expressions
}

fn find_subsequence(chars: &[char], needle: &[char], from: usize) -> Option<usize> {
    if from > chars.len() {
        return None;
    }
    chars[from..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|position| position + from)
}

/// True when a further `.` or `[` access immediately follows `start`
/// (after whitespace) — used to reject `needs.x.outputs.y.z`, which is not
/// a bare output reference.
pub(super) fn has_further_access(chars: &[char], start: usize) -> bool {
    let index = skip_whitespace(chars, start);
    matches!(chars.get(index), Some('.') | Some('['))
}
