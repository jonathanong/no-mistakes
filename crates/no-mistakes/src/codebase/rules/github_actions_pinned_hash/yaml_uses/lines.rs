use super::carry::Carry;
use super::indent::{in_input_block, is_ignorable, is_nested_key, is_value_line};

/// Job and step `uses` lines, in document order, plus whether each source line
/// opened inside a flow `with` or `env` or a block scalar. A root sequence
/// counts as steps. One physical line can hold several action refs.
#[derive(Debug)]
pub(super) struct ClassifiedUses {
    pub lines: Vec<usize>,
    pub flow_input_at_start: Vec<bool>,
    pub block_scalar: Vec<bool>,
    /// Nested `uses` keys that are not job or step actions. Line-scan hits on
    /// those lines are not action references.
    pub rejected_uses: Vec<bool>,
    /// Source line that holds the version comment for each action line.
    comment_lines: Vec<usize>,
}

struct OpenBlock {
    min_indent: usize,
    capture_uses: bool,
}

pub(super) fn action_uses_lines(lines: &[&str]) -> ClassifiedUses {
    let mut found = Vec::new();
    let mut flow_input_at_start = Vec::with_capacity(lines.len());
    let mut block_scalar = Vec::with_capacity(lines.len());
    let mut rejected_uses = Vec::with_capacity(lines.len());
    // `{ uses:` with the ref on the next line stays open across blanks.
    let mut await_value = false;
    // `uses:` then a comment or blank still owns the following scalar.
    let mut pending_value = false;
    // Index of an action whose quoted scalar is still open.
    let mut open_quote_at: Option<usize> = None;
    let mut comment_lines = Vec::new();
    let mut block: Option<OpenBlock> = None;
    let mut carry = Carry::default();
    for index in 0..lines.len() {
        if let Some(open) = block.as_mut() {
            if inside_block(lines[index], open.min_indent) {
                flow_input_at_start.push(false);
                block_scalar.push(true);
                rejected_uses.push(false);
                if open.capture_uses && is_value_line(lines[index]) && !is_nested_key(lines[index])
                {
                    record(&mut found, &mut comment_lines, index + 1);
                    open.capture_uses = false;
                }
                continue;
            }
            block = None;
        }
        let mark = carry.scan_line(lines[index]);
        flow_input_at_start.push(mark.starts_in_flow_input);
        block_scalar.push(false);
        rejected_uses.push(mark.rejects_uses);
        if in_input_block(lines, index) {
            await_value = false;
            pending_value = false;
            continue;
        }
        let value_here = is_value_line(lines[index]) && !is_nested_key(lines[index]);
        let ignorable = is_ignorable(lines[index]);
        // A rejected matrix `uses:` must not claim the next indented value.
        if !ignorable && (pending_value || await_value) && value_here && !mark.starts_in_flow_input
        {
            record(&mut found, &mut comment_lines, index + 1);
            pending_value = false;
            await_value = false;
        }
        let before = found.len();
        for _ in 0..mark.present_count {
            record(&mut found, &mut comment_lines, index + 1);
        }
        if mark.quote_open && found.len() > before {
            open_quote_at = Some(before);
        } else if let Some(slot) = open_quote_at {
            if !mark.quote_open {
                comment_lines[slot] = index + 1;
                open_quote_at = None;
            }
        }
        if mark.opens_block {
            block = Some(OpenBlock {
                min_indent: indent_of(lines[index]),
                capture_uses: mark.block_uses,
            });
        }
        if !ignorable {
            // `uses:` then `nested: true` is a mapping, not the action scalar.
            let bare_action = bare_uses_key(lines[index])
                && !mark.rejects_uses
                && mark.present_count == 0
                && !mark.opens_block
                && !mark.starts_in_flow_input;
            pending_value = bare_action;
            await_value = mark.awaits;
        }
    }
    ClassifiedUses {
        lines: found,
        flow_input_at_start,
        block_scalar,
        rejected_uses,
        comment_lines,
    }
}

fn record(found: &mut Vec<usize>, comments: &mut Vec<usize>, line: usize) {
    found.push(line);
    comments.push(line);
}

/// Version-comment line for an action. A block value's comment is on the header.
/// A quoted scalar folded with `\` keeps the action on the opening line and the
/// comment on the line that closes the quote.
pub(super) fn comment_at<'a>(
    classified: &ClassifiedUses,
    lines: &[&'a str],
    line_num: usize,
) -> &'a str {
    let mut index = classified
        .lines
        .iter()
        .position(|line| *line == line_num)
        .map(|slot| classified.comment_lines[slot].saturating_sub(1))
        .unwrap_or(line_num.saturating_sub(1));
    if classified.block_scalar.get(index).copied().unwrap_or(false) {
        while index > 0 && classified.block_scalar[index - 1] {
            index -= 1;
        }
        index = index.saturating_sub(1);
    }
    lines.get(index).copied().unwrap_or("")
}

fn bare_uses_key(line: &str) -> bool {
    let trimmed = line.trim();
    let trimmed = trimmed.strip_prefix('-').map(str::trim).unwrap_or(trimmed);
    matches!(trimmed, "uses:" | "\"uses\":" | "'uses':")
}

fn indent_of(line: &str) -> usize {
    line.chars().take_while(|ch| ch.is_whitespace()).count()
}

fn inside_block(line: &str, min_indent: usize) -> bool {
    line.trim().is_empty() || indent_of(line) > min_indent
}

#[cfg(test)]
mod tests;
