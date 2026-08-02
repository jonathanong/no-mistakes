use super::fence_syntax::{
    line_end_with_ending, markdown_column_after, strip_blockquote_prefix, FenceDelimiter,
};
use super::is_mermaid_info;

pub(super) fn opening_delimiter(
    source: &str,
    offset: usize,
    inside_list_item: bool,
) -> Option<FenceDelimiter> {
    let line_start = source[..offset.min(source.len())]
        .rfind(['\n', '\r'])
        .map_or(0, |index| index + 1);
    let line_end = source[line_start..]
        .find(['\n', '\r'])
        .map_or(source.len(), |index| line_start + index);
    let (line, blockquote_depth, line_start_column) =
        strip_blockquote_prefix(&source.as_bytes()[line_start..line_end]);

    for index in 0..line.len() {
        let marker = line[index];
        if marker != b'`' && marker != b'~' {
            continue;
        }
        let length = line[index..]
            .iter()
            .take_while(|byte| **byte == marker)
            .count();
        if length < 3 {
            continue;
        }
        // The slice begins after an ASCII fence-marker run inside an existing
        // UTF-8 string, so both boundaries are necessarily valid.
        let trailing = std::str::from_utf8(&line[index + length..])
            .expect("slice after an ASCII fence marker must remain UTF-8");
        if is_mermaid_info(trailing) {
            let prefix = &line[..index];
            let container_indent = if prefix.iter().any(|byte| !byte.is_ascii_whitespace())
                || (inside_list_item && !prefix.is_empty())
            {
                markdown_column_after(line_start_column, prefix) - line_start_column
            } else {
                0
            };
            return Some(FenceDelimiter {
                marker,
                length,
                blockquote_depth,
                container_indent,
                content_start: line_end_with_ending(source, line_end),
            });
        }
    }
    None
}

pub(super) fn line_number(source: &str, offset: usize) -> usize {
    let bytes = source.as_bytes();
    let mut line = 1;
    let mut index = 0;
    let limit = offset.min(bytes.len());
    while index < limit {
        match bytes[index] {
            b'\n' => line += 1,
            b'\r' => {
                line += 1;
                if bytes.get(index + 1) == Some(&b'\n') {
                    index += 1;
                }
            }
            _ => {}
        }
        index += 1;
    }
    line
}
