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
}

struct OpenBlock {
    min_indent: usize,
    capture_uses: bool,
}

pub(super) fn action_uses_lines(lines: &[&str]) -> ClassifiedUses {
    let mut found = Vec::new();
    let mut flow_input_at_start = Vec::with_capacity(lines.len());
    let mut block_scalar = Vec::with_capacity(lines.len());
    // `{ uses:` with the ref on the next line stays open across blanks.
    let mut await_value = false;
    let mut block: Option<OpenBlock> = None;
    let mut carry = Carry::default();
    for index in 0..lines.len() {
        if let Some(open) = block.as_mut() {
            if inside_block(lines[index], open.min_indent) {
                flow_input_at_start.push(false);
                block_scalar.push(true);
                if open.capture_uses && is_value_line(lines[index]) && !is_nested_key(lines[index])
                {
                    found.push(index + 1);
                    open.capture_uses = false;
                }
                continue;
            }
            block = None;
        }
        let mark = carry.scan_line(lines[index]);
        flow_input_at_start.push(mark.starts_in_flow_input);
        block_scalar.push(false);
        if in_input_block(lines, index) {
            await_value = false;
            continue;
        }
        let value_here = is_value_line(lines[index]) && !is_nested_key(lines[index]);
        let previous_in_flow = index > 0 && flow_input_at_start[index - 1];
        // `uses:` then `nested: true` is a mapping, not the action scalar.
        let continued = index > 0
            && bare_uses_key(lines[index - 1])
            && !in_input_block(lines, index - 1)
            && !previous_in_flow
            && value_here
            && !mark.starts_in_flow_input;
        if continued || (await_value && value_here && !mark.starts_in_flow_input) {
            found.push(index + 1);
        }
        for _ in 0..mark.present_count {
            found.push(index + 1);
        }
        if mark.opens_block {
            block = Some(OpenBlock {
                min_indent: indent_of(lines[index]),
                capture_uses: mark.block_uses,
            });
        }
        if !is_ignorable(lines[index]) {
            await_value = mark.awaits;
        }
    }
    ClassifiedUses {
        lines: found,
        flow_input_at_start,
        block_scalar,
    }
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
