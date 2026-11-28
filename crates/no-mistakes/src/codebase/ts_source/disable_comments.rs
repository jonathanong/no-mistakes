const DISABLE_FILE: &str = "no-mistakes-disable-file ";
const DISABLE_LINE: &str = "no-mistakes-disable-line ";
const DISABLE_NEXT_LINE: &str = "no-mistakes-disable-next-line ";

pub use no_mistakes_ts_analysis::text::byte_offset_to_line;

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
        .and_then(next_line_rule_part)
        .is_some_and(|part| rule_part_matches(part, rule_id))
}

fn next_line_rule_part(line: &str) -> Option<&str> {
    leading_comment_text(line.trim())?
        .strip_prefix(DISABLE_NEXT_LINE)
        .map(str::trim)
}

/// Returns `true` if `stmt_line` (1-based) contains a
/// `no-mistakes-disable-line <rule_id>` directive comment.
///
/// Matches:
/// - `// no-mistakes-disable-line <rule_id>`
/// - `// no-mistakes-disable-line <rule_id>: <reason>`
/// - `// no-mistakes-disable-line <rule_id> <reason>`
pub fn has_disable_line_comment(source: &str, stmt_line: u32, rule_id: &str) -> bool {
    directives::has_disable_line_comment_tracking(source, stmt_line, rule_id, true)
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
    disable_file_directive_line_at(source, rule_id, None)
}

fn disable_file_directive_line_at(
    source: &str,
    rule_id: &str,
    expected: Option<u32>,
) -> Option<u32> {
    scan_file_directives(source, |line, part| {
        expected.is_none_or(|expected| expected == line) && rule_part_matches(part, rule_id)
    })
}

fn scan_file_directives(source: &str, mut accept: impl FnMut(u32, &str) -> bool) -> Option<u32> {
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
            let Some(after_directive) = rest.strip_prefix(DISABLE_FILE) else {
                break;
            };
            if saw_hash_attribute && comment_prefix_is_slash {
                return None;
            }
            let line = (index + 1) as u32;
            if accept(line, after_directive.trim()) {
                return Some(line);
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

fn rule_part_matches(rule_part: &str, rule_id: &str) -> bool {
    rule_part.strip_prefix(rule_id).is_some_and(|suffix| {
        suffix.is_empty() || suffix.starts_with(':') || suffix.starts_with(char::is_whitespace)
    })
}
fn named_rule(rule_part: &str) -> &str {
    rule_part
        .split(|ch: char| ch == ':' || ch.is_whitespace())
        .next()
        .unwrap_or("")
}

#[path = "disable_comments/index.rs"]
mod disable_comment_index;
pub(crate) use disable_comment_index::DisableCommentIndex;

#[path = "disable_comments/directives.rs"]
mod directives;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
use directives::{has_disable_line_comment_tracking, matching_disable_directive_tracking};
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
    if !source.contains(DISABLE_LINE) && !source.contains(DISABLE_NEXT_LINE) {
        return Vec::new();
    }
    let mut state = LineCommentScanState {
        track_quotes: true,
        ..LineCommentScanState::default()
    };
    let mut next_disabled = false;
    let mut line_count = 0;
    let mut disabled = Vec::new();
    for (index, line) in source.trim_start_matches('\u{FEFF}').lines().enumerate() {
        line_count = index + 1;
        scanned_line();
        let inline_disabled = directives::inline_rule_part(line, &mut state)
            .is_some_and(|part| rule_part_matches(part, rule_id));
        if next_disabled || inline_disabled {
            disabled.push((index + 1) as u32);
        }
        next_disabled =
            next_line_rule_part(line).is_some_and(|part| rule_part_matches(part, rule_id));
    }
    // Preserve next-line queries for an otherwise empty final line as well.
    if next_disabled {
        disabled.push((line_count + 1) as u32);
    }
    disabled
}

#[path = "disable_comments/inventory.rs"]
mod directive_inventory;
#[doc(hidden)]
pub use directive_inventory::{disable_directives, NamedDisableDirective};

#[path = "disable_comments/path_inventory.rs"]
mod path_inventory;
pub(crate) use path_inventory::audit_directives_for_path;
