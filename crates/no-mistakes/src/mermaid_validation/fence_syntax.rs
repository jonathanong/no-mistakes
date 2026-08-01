#[derive(Debug, Clone, Copy)]
pub(super) struct FenceDelimiter {
    pub(super) marker: u8,
    pub(super) length: usize,
    pub(super) blockquote_depth: usize,
    pub(super) container_indent: usize,
    pub(super) content_start: usize,
}

pub(super) fn has_closing_fence(source: &str, delimiter: FenceDelimiter, block_end: usize) -> bool {
    let mut cursor = delimiter.content_start;
    let limit = block_end.min(source.len());
    while cursor < limit {
        let relative_end = source[cursor..limit]
            .find(['\n', '\r'])
            .unwrap_or(limit - cursor);
        let line_end = cursor + relative_end;
        if is_closing_fence_line(
            &source.as_bytes()[cursor..line_end],
            delimiter.marker,
            delimiter.length,
            delimiter.blockquote_depth,
            delimiter.container_indent,
        ) {
            return true;
        }
        cursor = line_end_with_ending(source, line_end);
    }
    false
}

fn is_closing_fence_line(
    line: &[u8],
    marker: u8,
    opening_length: usize,
    opening_blockquote_depth: usize,
    container_indent: usize,
) -> bool {
    let (line, blockquote_depth, line_start_column) = strip_blockquote_prefix(line);
    if blockquote_depth != opening_blockquote_depth {
        return false;
    }
    let (indent_bytes, indent_columns) = leading_indentation(line, line_start_column);
    if indent_columns > container_indent + 3 {
        return false;
    }
    let remainder = &line[indent_bytes..];
    let marker_length = remainder.iter().take_while(|byte| **byte == marker).count();
    marker_length >= opening_length && is_closing_fence_suffix(&remainder[marker_length..])
}

fn leading_indentation(line: &[u8], start_column: usize) -> (usize, usize) {
    let bytes = line
        .iter()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    (
        bytes,
        markdown_column_after(start_column, &line[..bytes]) - start_column,
    )
}

pub(super) fn markdown_column_after(start_column: usize, bytes: &[u8]) -> usize {
    bytes.iter().fold(start_column, |columns, byte| match byte {
        b'\t' => columns + 4 - (columns % 4),
        _ => columns + 1,
    })
}

pub(super) fn strip_blockquote_prefix(mut line: &[u8]) -> (&[u8], usize, usize) {
    let mut depth = 0;
    let mut column = 0;
    loop {
        let spaces = line.iter().take_while(|byte| **byte == b' ').count();
        if spaces > 3 || line.get(spaces) != Some(&b'>') {
            return (line, depth, column);
        }
        depth += 1;
        column += spaces + 1;
        line = &line[spaces + 1..];
        if line.first() == Some(&b' ') {
            line = &line[1..];
            column += 1;
        }
    }
}

pub(super) fn is_closing_fence_suffix(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| matches!(byte, b' ' | b'\t'))
}

pub(super) fn line_end_with_ending(source: &str, line_end: usize) -> usize {
    match source.as_bytes().get(line_end) {
        Some(b'\r') if source.as_bytes().get(line_end + 1) == Some(&b'\n') => line_end + 2,
        Some(b'\r' | b'\n') => line_end + 1,
        _ => line_end,
    }
}

#[cfg(test)]
#[path = "fence_syntax/tests.rs"]
mod tests;
