use super::super::{is_inline_value_line, yaml_key_value};
use super::{
    block::{explicit_key_name, explicit_value},
    flow_sequence_item_lines, has_child_inline_value, indent, is_content, sequence_item_lines,
};

fn find_parallel_key(lines: &[&str], parent: usize) -> Option<(usize, usize)> {
    let parent_line = *lines.get(parent)?;
    let parent_depth = indent(parent_line);
    if parent_line.trim_start().starts_with("- ")
        && yaml_key_value(parent_line).is_some_and(|(key, _)| key == "parallel")
    {
        return Some((parent, parent_depth + 2));
    }
    let mut explicit_parallel = explicit_key_name(parent_line) == Some("parallel");
    let mut child_indent = parent_line
        .trim_start()
        .strip_prefix("- ")
        .map(|rest| parent_depth + 2 + rest.len() - rest.trim_start_matches(' ').len());
    for (index, line) in lines.iter().enumerate().skip(parent + 1) {
        if !is_content(line) {
            continue;
        }
        let depth = indent(line);
        if depth <= parent_depth {
            break;
        }
        let direct_depth = *child_indent.get_or_insert(depth);
        if depth == direct_depth {
            if yaml_key_value(line).is_some_and(|(key, _)| key == "parallel")
                || (explicit_parallel && explicit_value(line))
            {
                return Some((index, depth));
            }
            explicit_parallel = explicit_key_name(line) == Some("parallel");
        }
    }
    None
}

pub(super) fn parallel_child_lines(lines: &[&str], parent_line: usize, count: usize) -> Vec<usize> {
    let Some(key) = parent_line
        .checked_sub(1)
        .and_then(|parent| find_parallel_key(lines, parent))
    else {
        return vec![1; count];
    };
    if is_inline_value_line(lines[key.0]) || has_child_inline_value(lines, key, '[') {
        return flow_sequence_item_lines(lines, key, count);
    }
    sequence_item_lines(lines, key, count)
}
