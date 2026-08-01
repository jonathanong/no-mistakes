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
    let (line, blockquote_depth) = strip_blockquote_prefix(line);
    if blockquote_depth != opening_blockquote_depth {
        return false;
    }
    let (indent_bytes, indent_columns) = leading_indentation(line);
    if indent_columns > container_indent + 3 {
        return false;
    }
    let remainder = &line[indent_bytes..];
    let marker_length = remainder.iter().take_while(|byte| **byte == marker).count();
    marker_length >= opening_length
        && remainder[marker_length..]
            .iter()
            .all(|byte| byte.is_ascii_whitespace())
}

fn leading_indentation(line: &[u8]) -> (usize, usize) {
    let mut bytes = 0;
    let mut columns = 0;
    for byte in line {
        match byte {
            b' ' => columns += 1,
            b'\t' => columns += 4 - (columns % 4),
            _ => break,
        }
        bytes += 1;
    }
    (bytes, columns)
}

pub(super) fn strip_blockquote_prefix(mut line: &[u8]) -> (&[u8], usize) {
    let mut depth = 0;
    loop {
        let spaces = line.iter().take_while(|byte| **byte == b' ').count();
        if spaces > 3 || line.get(spaces) != Some(&b'>') {
            return (line, depth);
        }
        depth += 1;
        line = &line[spaces + 1..];
        if line.first() == Some(&b' ') {
            line = &line[1..];
        }
    }
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
