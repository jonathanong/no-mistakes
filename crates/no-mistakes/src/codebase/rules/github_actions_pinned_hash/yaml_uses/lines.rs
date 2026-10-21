use super::carry::{Carry, LineKind};
use super::indent::{in_input_block, is_ignorable, is_nested_key, is_value_line};

/// Job and step `uses` lines, in document order, plus whether each source line
/// opened inside a flow `with` or `env`. A root sequence counts as steps.
pub(super) struct ClassifiedUses {
    pub lines: Vec<usize>,
    pub flow_input_at_start: Vec<bool>,
}

pub(super) fn action_uses_lines(lines: &[&str]) -> ClassifiedUses {
    let mut found = Vec::new();
    let mut flow_input_at_start = Vec::with_capacity(lines.len());
    // `{ uses:` with the ref on the next line stays open across blanks.
    let mut await_value = false;
    let mut carry = Carry::default();
    for index in 0..lines.len() {
        let mark = carry.scan_line(lines[index]);
        flow_input_at_start.push(mark.starts_in_flow_input);
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
        let in_flow_input = mark.starts_in_flow_input;
        if mark.kind == LineKind::Present
            || continued
            || (await_value && value_here && !in_flow_input)
        {
            found.push(index + 1);
        }
        if !is_ignorable(lines[index]) {
            await_value = mark.kind == LineKind::Awaits;
        }
    }
    ClassifiedUses {
        lines: found,
        flow_input_at_start,
    }
}

fn bare_uses_key(line: &str) -> bool {
    let trimmed = line.trim();
    let trimmed = trimmed.strip_prefix('-').map(str::trim).unwrap_or(trimmed);
    matches!(trimmed, "uses:" | "\"uses\":" | "'uses':")
}

#[cfg(test)]
mod classify_tests {
    use super::action_uses_lines;

    #[test]
    fn flow_uses_on_one_line_is_an_action_line() {
        let classified = action_uses_lines(&["- { uses: actions/checkout@v4 }"]);
        assert_eq!(
            classified.lines,
            vec![1],
            "{:?}",
            classified.flow_input_at_start
        );
    }
}
