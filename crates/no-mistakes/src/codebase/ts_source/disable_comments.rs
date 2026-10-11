pub fn byte_offset_to_line(source: &str, byte_offset: usize) -> u32 {
    let end = byte_offset.min(source.len());
    let line = source[..end].bytes().filter(|&b| b == b'\n').count();
    (line + 1) as u32
}

/// Returns `true` if the line immediately before `stmt_line` (1-based) contains
/// a `no-mistakes-disable-next-line <rule_id>` directive comment.
///
/// Matches:
/// - `// no-mistakes-disable-next-line <rule_id>`
/// - `// no-mistakes-disable-next-line <rule_id>: <reason>`
/// - `// no-mistakes-disable-next-line <rule_id> <reason>`
pub fn has_disable_comment(source: &str, stmt_line: u32, rule_id: &str) -> bool {
    if stmt_line < 2 {
        return false;
    }
    source
        .trim_start_matches('\u{FEFF}')
        .lines()
        .nth((stmt_line - 2) as usize)
        .map(|line| {
            let trimmed = line.trim();
            let Some(rest) = leading_comment_text(trimmed) else {
                return false;
            };
            let Some(after_directive) = rest.strip_prefix("no-mistakes-disable-next-line ") else {
                return false;
            };
            rule_part_matches(after_directive.trim(), rule_id)
        })
        .unwrap_or(false)
}

/// Returns `true` if `stmt_line` (1-based) contains a
/// `no-mistakes-disable-line <rule_id>` directive comment.
///
/// Matches:
/// - `// no-mistakes-disable-line <rule_id>`
/// - `// no-mistakes-disable-line <rule_id>: <reason>`
/// - `// no-mistakes-disable-line <rule_id> <reason>`
pub fn has_disable_line_comment(source: &str, stmt_line: u32, rule_id: &str) -> bool {
    if stmt_line == 0 {
        return false;
    }
    line_comment_directive_matches(source, stmt_line, "no-mistakes-disable-line ", rule_id)
}

/// Returns `true` if a leading comment disables `rule_id` for the whole file.
///
/// Matches:
/// - `// no-mistakes-disable-file <rule_id>`
/// - `// no-mistakes-disable-file <rule_id>: <reason>`
/// - `// no-mistakes-disable-file <rule_id> <reason>`
pub fn has_disable_file_comment(source: &str, rule_id: &str) -> bool {
    disable_file_directive_line(source, rule_id).is_some()
}

fn disable_file_directive_line(source: &str, rule_id: &str) -> Option<u32> {
    let mut in_block_comment = false;
    let mut saw_hash_attribute = false;

    for (index, line) in source.trim_start_matches('\u{FEFF}').lines().enumerate() {
        let mut rest = line.trim();

        loop {
            if rest.is_empty() {
                break;
            }

            if in_block_comment {
                let Some(end) = rest.find("*/") else {
                    break;
                };
                in_block_comment = false;
                rest = rest[end + 2..].trim();
                continue;
            }

            if rest.starts_with("/*") {
                let Some(end) = rest.find("*/") else {
                    in_block_comment = true;
                    break;
                };
                rest = rest[end + 2..].trim();
                continue;
            }

            let comment_prefix_is_slash = rest.starts_with("//");
            saw_hash_attribute |= hash_attribute_comment_line(rest);
            let rest = leading_comment_text(rest)?;
            let Some(after_directive) = rest.strip_prefix("no-mistakes-disable-file ") else {
                break;
            };
            if saw_hash_attribute && comment_prefix_is_slash {
                return None;
            }
            let rule_part = after_directive.trim();
            if rule_part_matches(rule_part, rule_id) {
                return Some((index + 1) as u32);
            }
            break;
        }
    }

    None
}

fn hash_attribute_comment_line(line: &str) -> bool {
    line.starts_with("#![")
        || line
            .strip_prefix("#[")
            .and_then(|rest| rest.chars().next())
            .is_some_and(is_word_char)
}

fn line_comment_directive_matches(
    source: &str,
    stmt_line: u32,
    directive: &str,
    rule_id: &str,
) -> bool {
    let mut state = LineCommentScanState::default();
    for (index, line) in source.trim_start_matches('\u{FEFF}').lines().enumerate() {
        let line_number = (index + 1) as u32;
        let comment = line_comment_start(line, &mut state);

        if line_number != stmt_line {
            continue;
        }

        let Some((comment_start, prefix_len)) = comment else {
            return false;
        };
        let rest = line[comment_start + prefix_len..].trim();
        let Some(after_directive) = rest.strip_prefix(directive) else {
            return false;
        };
        return rule_part_matches(after_directive.trim(), rule_id);
    }

    false
}

fn rule_part_matches(rule_part: &str, rule_id: &str) -> bool {
    rule_part.strip_prefix(rule_id).is_some_and(|suffix| {
        suffix.is_empty() || suffix.starts_with(':') || suffix.starts_with(char::is_whitespace)
    })
}
#[path = "disable_comments/directives.rs"]
mod directives;
pub use directives::{matching_disable_directive, DisableDirective};

/// Index line suppressions in one lexical pass. Next-line directives deliberately
/// retain the leading-comment semantics of `has_disable_comment`.
pub(crate) fn disabled_statement_lines(source: &str, rule_id: &str) -> Vec<u32> {
    disabled_statement_lines_with_scan_observer(source, rule_id, || {})
}

// The no-op observer optimizes away in production; tests observe the actual
// scanner loop without conditional instrumentation in source code.
fn disabled_statement_lines_with_scan_observer(
    source: &str,
    rule_id: &str,
    mut scanned_line: impl FnMut(),
) -> Vec<u32> {
    if !source.contains("no-mistakes-disable-line ")
        && !source.contains("no-mistakes-disable-next-line ")
    {
        return Vec::new();
    }
    let mut state = LineCommentScanState::default();
    let mut next_disabled = false;
    let mut line_count = 0;
    let mut disabled = Vec::new();
    for (index, line) in source.trim_start_matches('\u{FEFF}').lines().enumerate() {
        line_count = index + 1;
        scanned_line();
        let inline_disabled = line_comment_start(line, &mut state)
            .and_then(|(start, len)| line[start + len..].trim().strip_prefix("no-mistakes-disable-line "))
            .is_some_and(|rest| rule_part_matches(rest.trim(), rule_id));
        if next_disabled || inline_disabled {
            disabled.push((index + 1) as u32);
        }
        next_disabled = leading_comment_text(line.trim())
            .and_then(|rest| rest.strip_prefix("no-mistakes-disable-next-line "))
            .is_some_and(|rest| rule_part_matches(rest.trim(), rule_id));
    }
    // Preserve next-line queries for an otherwise empty final line as well.
    if next_disabled {
        disabled.push((line_count + 1) as u32);
    }
    disabled
}
