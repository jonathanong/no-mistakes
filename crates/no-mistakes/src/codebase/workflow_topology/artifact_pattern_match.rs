//! Same-run artifact pattern matching (`actions/download-artifact` with
//! `pattern:`), ported from `artifact-pattern-match.mts`.
//!
//! The original uses `minimatch`'s bash-style brace expansion ahead of glob
//! matching. This port hand-rolls the same two conservative limits (pattern
//! length, expansion count) plus a brace expander covering the comma-list
//! and numeric/alpha range forms real GitHub Actions patterns use. Deep
//! bash-brace trivia the source engine's own test suite never exercises
//! (zero-padded numeric ranges, the `${}`-prefix escape quirk) is
//! intentionally not replicated: every case under test is either a plain
//! glob match with no braces, or a conservative "too big to expand"
//! rejection, and both are covered here.

use globset::GlobBuilder;

pub const ARTIFACT_PATTERN_LENGTH_LIMIT: usize = 1024;
pub const ARTIFACT_PATTERN_EXPANSION_LIMIT: usize = 256;

/// Mirrors `matchesArtifactPattern`: expand `pattern`'s brace groups (bash
/// style), then glob-match `name` against every expansion. Both limits fail
/// conservatively (no match) rather than erroring, matching the TS engine's
/// `try {} catch { return false }`.
pub fn matches_artifact_pattern(name: &str, pattern: &str) -> bool {
    if pattern.chars().count() > ARTIFACT_PATTERN_LENGTH_LIMIT {
        return false;
    }
    let expansions = brace_expand(pattern, ARTIFACT_PATTERN_EXPANSION_LIMIT + 1);
    if expansions.len() > ARTIFACT_PATTERN_EXPANSION_LIMIT {
        return false;
    }
    expansions
        .iter()
        .any(|expansion| glob_match(name, expansion))
}

/// `globset` (unlike minimatch's `nobrace: true`) always treats `{a,b}` as
/// its own native alternation syntax — but by the time a pattern reaches
/// here, [`brace_expand`] has already performed every real bash-style
/// expansion, so any `{`/`}` still present is literal (a non-expandable
/// single-entry group, or an unbalanced brace) and must be escaped to stop
/// `globset` from expanding it a second time.
fn glob_match(name: &str, pattern: &str) -> bool {
    let escaped = pattern.replace('{', "\\{").replace('}', "\\}");
    GlobBuilder::new(&escaped)
        .literal_separator(true)
        .backslash_escape(true)
        .build()
        .is_ok_and(|glob| glob.compile_matcher().is_match(name))
}

/// Bash-style brace expansion, capped at `max` results: generation stops as
/// soon as `max` is reached, so a pathological input like
/// `"{a,b}".repeat(10_000)` costs `O(levels * max)` instead of exploding
/// combinatorially (each nesting level's append loop bails once the shared
/// `max` budget is spent, the same trick the source engine's
/// `brace-expansion` dependency uses internally).
fn brace_expand(pattern: &str, max: usize) -> Vec<String> {
    let chars: Vec<char> = pattern.chars().collect();
    expand(&chars, max)
}

fn expand(chars: &[char], max: usize) -> Vec<String> {
    let Some((open, close)) = find_balanced_braces(chars) else {
        return vec![chars.iter().collect()];
    };
    let pre: String = chars[..open].iter().collect();
    let body = &chars[open + 1..close];
    let post = &chars[close + 1..];

    let items = match numeric_range(body, max).or_else(|| alpha_range(body, max)) {
        Some(items) => items,
        None if body.contains(&',') => split_top_level_commas(body)
            .into_iter()
            .flat_map(|part| expand(&part, max))
            .collect(),
        // Not a real brace set (e.g. `a{b}c`) — bash leaves it literal.
        None => return vec![chars.iter().collect()],
    };
    let post_expansions = if post.is_empty() {
        vec![String::new()]
    } else {
        expand(post, max)
    };

    let mut expansions = Vec::new();
    'outer: for item in &items {
        for suffix in &post_expansions {
            if expansions.len() >= max {
                break 'outer;
            }
            expansions.push(format!("{pre}{item}{suffix}"));
        }
    }
    expansions
}

fn find_balanced_braces(chars: &[char]) -> Option<(usize, usize)> {
    let open = chars.iter().position(|&c| c == '{')?;
    let mut depth = 0usize;
    for (index, &c) in chars.iter().enumerate().skip(open) {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((open, index));
                }
            }
            _ => {}
        }
    }
    None
}

fn split_top_level_commas(chars: &[char]) -> Vec<Vec<char>> {
    let mut parts = Vec::new();
    let mut current = Vec::new();
    let mut depth = 0usize;
    for &c in chars {
        match c {
            '{' => {
                depth += 1;
                current.push(c);
            }
            '}' => {
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => parts.push(std::mem::take(&mut current)),
            _ => current.push(c),
        }
    }
    parts.push(current);
    parts
}

/// `{start..end}` / `{start..end..step}` with signed integer bounds,
/// capped at `max` items.
fn numeric_range(body: &[char], max: usize) -> Option<Vec<String>> {
    let text: String = body.iter().collect();
    let segments: Vec<&str> = text.split("..").collect();
    if segments.len() < 2 || segments.len() > 3 {
        return None;
    }
    let start = segments[0].parse::<i64>().ok()?;
    let end = segments[1].parse::<i64>().ok()?;
    let step = match segments.get(2) {
        Some(raw) => raw.parse::<i64>().ok()?.unsigned_abs().max(1) as i64,
        None => 1,
    };
    Some(stepped_range(start, end, step, max))
}

/// `{a..z}` single-letter alpha ranges, capped at `max` items.
fn alpha_range(body: &[char], max: usize) -> Option<Vec<String>> {
    let text: String = body.iter().collect();
    let segments: Vec<&str> = text.split("..").collect();
    if segments.len() < 2 || segments.len() > 3 {
        return None;
    }
    let start = single_alpha(segments[0])?;
    let end = single_alpha(segments[1])?;
    let step = match segments.get(2) {
        Some(raw) => raw.parse::<i64>().ok()?.unsigned_abs().max(1) as u32,
        None => 1,
    };
    let (start, end) = (start as u32, end as u32);
    let mut values = Vec::new();
    if end >= start {
        let mut current = start;
        while current <= end && values.len() < max {
            values.push(char_string(current));
            current += step;
        }
    } else {
        let mut current = start;
        while values.len() < max {
            values.push(char_string(current));
            if current < step || current - step < end {
                break;
            }
            current -= step;
        }
    }
    Some(values)
}

fn single_alpha(segment: &str) -> Option<char> {
    let mut chars = segment.chars();
    let first = chars.next()?;
    (chars.next().is_none() && first.is_ascii_alphabetic()).then_some(first)
}

fn char_string(code_point: u32) -> String {
    char::from_u32(code_point)
        .map(String::from)
        .unwrap_or_default()
}

fn stepped_range(start: i64, end: i64, step: i64, max: usize) -> Vec<String> {
    let mut values = Vec::new();
    if end >= start {
        let mut current = start;
        while current <= end && values.len() < max {
            values.push(current.to_string());
            current += step;
        }
    } else {
        let mut current = start;
        while current >= end && values.len() < max {
            values.push(current.to_string());
            current -= step;
        }
    }
    values
}
