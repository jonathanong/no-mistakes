//! `no-mistakes-bound <rule> rows<=N: <justification>` line comments.
//!
//! The scanner is the disable-directive scanner, so a copy of the directive
//! inside a string or template is not a declaration.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BoundComment {
    Valid {
        line: u32,
        rows: u64,
        justification: String,
    },
    Invalid {
        line: u32,
        error: BoundCommentError,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoundCommentError {
    MissingBound,
    MissingJustification,
}

pub(crate) fn bound_comments(source: &str, rule_id: &str) -> Vec<BoundComment> {
    if !source.contains("no-mistakes-bound") {
        return Vec::new();
    }
    let mut state = super::LineCommentScanState {
        track_quotes: true,
        ..super::LineCommentScanState::default()
    };
    let mut found = Vec::new();
    for (index, line) in source.trim_start_matches('\u{FEFF}').lines().enumerate() {
        let Some((start, kind)) = super::line_comment_start(line, &mut state) else {
            continue;
        };
        let Some(payload) = directive_payload(super::comment_body(line, start, kind), rule_id)
        else {
            continue;
        };
        found.push(parse_payload((index + 1) as u32, payload));
    }
    found.extend(template_sql_comments(source, rule_id));
    found.sort_by_key(comment_line);
    found
}

fn comment_line(comment: &BoundComment) -> u32 {
    match comment {
        BoundComment::Valid { line, .. } | BoundComment::Invalid { line, .. } => *line,
    }
}

/// `--` inside a template is SQL, so quote tracking must not hide it.
/// A copy inside `'` or `"` stays hidden.
fn template_sql_comments(source: &str, rule_id: &str) -> Vec<BoundComment> {
    let mut found = Vec::new();
    let mut template = false;
    let mut braces = 0u32;
    let mut sql_string = false;
    for (index, line) in source.trim_start_matches('\u{FEFF}').lines().enumerate() {
        let Some(body) = template_directive(line, &mut template, &mut braces, &mut sql_string)
        else {
            continue;
        };
        let Some(payload) = directive_payload(body, rule_id) else {
            continue;
        };
        found.push(parse_payload((index + 1) as u32, payload));
    }
    found
}

fn template_directive<'a>(
    line: &'a str,
    template: &mut bool,
    braces: &mut u32,
    sql_string: &mut bool,
) -> Option<&'a str> {
    let mut escaped = false;
    let mut chars = line.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        if *braces > 0 {
            if ch == '{' {
                *braces += 1;
            } else if ch == '}' {
                *braces -= 1;
                if *braces == 0 {
                    *template = true;
                }
            }
            continue;
        }
        if !*template {
            if ch == '`' {
                *template = true;
            }
            continue;
        }
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if *sql_string {
            if ch == '\'' {
                if chars.peek().is_some_and(|(_, next)| *next == '\'') {
                    chars.next();
                } else {
                    *sql_string = false;
                }
            }
            continue;
        }
        if ch == '\'' {
            *sql_string = true;
            continue;
        }
        if ch == '`' {
            *template = false;
            continue;
        }
        if ch == '$' && chars.peek().is_some_and(|(_, next)| *next == '{') {
            chars.next();
            *braces = 1;
            *template = false;
            continue;
        }
        if ch == '-' && super::dash_comment_can_start(line, idx) {
            return Some(line[idx + 2..].trim());
        }
    }
    None
}

fn directive_payload<'a>(body: &'a str, rule_id: &str) -> Option<&'a str> {
    let rest = body.strip_prefix("no-mistakes-bound")?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let after = rest.trim_start();
    if !super::rule_part_matches(after, rule_id) {
        return None;
    }
    Some(after[rule_id.len()..].trim_start())
}

fn parse_payload(line: u32, payload: &str) -> BoundComment {
    let Some(rest) = payload.strip_prefix("rows<=") else {
        return invalid(line, BoundCommentError::MissingBound);
    };
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return invalid(line, BoundCommentError::MissingBound);
    }
    let (number, after) = rest.split_at(digits);
    let Ok(rows) = number.parse::<u64>() else {
        return invalid(line, BoundCommentError::MissingBound);
    };
    let Some(justification) = after.trim_start().strip_prefix(':') else {
        return invalid(line, BoundCommentError::MissingJustification);
    };
    let justification = justification.trim();
    if justification.is_empty() {
        return invalid(line, BoundCommentError::MissingJustification);
    }
    BoundComment::Valid {
        line,
        rows,
        justification: justification.to_string(),
    }
}

fn invalid(line: u32, error: BoundCommentError) -> BoundComment {
    BoundComment::Invalid { line, error }
}

#[cfg(test)]
#[path = "bound_comments_tests.rs"]
mod tests;
