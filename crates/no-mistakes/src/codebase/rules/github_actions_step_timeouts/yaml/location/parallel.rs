use super::super::{is_inline_value_line, yaml_key_value};
use super::{indent, is_content, sequence_item_line};

fn find_parallel_key(lines: &[&str], parent: usize) -> Option<(usize, usize)> {
    let parent_line = *lines.get(parent)?;
    let parent_depth = indent(parent_line);
    if parent_line.trim_start().starts_with("- ")
        && yaml_key_value(parent_line).is_some_and(|(key, _)| key == "parallel")
    {
        return Some((parent, parent_depth + 2));
    }
    let mut child_indent = None;
    for (index, line) in lines.iter().enumerate().skip(parent + 1) {
        if !is_content(line) {
            continue;
        }
        let depth = indent(line);
        if depth <= parent_depth {
            break;
        }
        let direct_depth = *child_indent.get_or_insert(depth);
        if depth == direct_depth && yaml_key_value(line).is_some_and(|(key, _)| key == "parallel") {
            return Some((index, depth));
        }
    }
    None
}

pub(super) fn parallel_child_line(source: &str, parent_line: usize, index: usize) -> Option<usize> {
    let lines: Vec<_> = source.lines().collect();
    let key = find_parallel_key(&lines, parent_line.checked_sub(1)?)?;
    if is_inline_value_line(lines[key.0]) {
        return Some(key.0 + 1);
    }
    sequence_item_line(&lines, key, index)
}
