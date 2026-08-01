#[derive(Clone, Copy, Default)]
pub(super) struct ContainerPrefix {
    blockquote_depth: usize,
    list_indent: Option<usize>,
}

impl ContainerPrefix {
    pub(super) fn from_opening_line(mut line: &[u8]) -> (&[u8], Self) {
        let mut blockquote_depth = 0;
        while let Some(remainder) = strip_one_blockquote(line) {
            blockquote_depth += 1;
            line = remainder;
        }
        let mut list_indent = None;
        while let Some((remainder, indent)) = strip_one_list_marker(line) {
            list_indent = Some(list_indent.unwrap_or(0) + indent);
            line = remainder;
        }
        (
            line,
            Self {
                blockquote_depth,
                list_indent,
            },
        )
    }

    pub(super) fn strip_line<'line>(&self, mut line: &'line [u8]) -> Option<&'line [u8]> {
        if line.iter().all(|byte| byte.is_ascii_whitespace()) {
            return Some(&line[line.len()..]);
        }
        for _ in 0..self.blockquote_depth {
            line = strip_one_blockquote(line)?;
        }
        match self.list_indent {
            Some(indent) => strip_indentation(line, indent),
            None => Some(line),
        }
    }
}

fn strip_one_blockquote(line: &[u8]) -> Option<&[u8]> {
    let spaces = line.iter().take_while(|byte| **byte == b' ').count();
    if spaces > 3 || line.get(spaces) != Some(&b'>') {
        return None;
    }
    let mut end = spaces + 1;
    if line
        .get(end)
        .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
    {
        end += 1;
    }
    Some(&line[end..])
}

fn strip_one_list_marker(line: &[u8]) -> Option<(&[u8], usize)> {
    let leading = line.iter().take_while(|byte| **byte == b' ').count();
    if leading > 3 {
        return None;
    }
    let marker_end = match line.get(leading)? {
        b'-' | b'+' | b'*' => leading + 1,
        byte if byte.is_ascii_digit() => ordered_marker_end(line, leading)?,
        _ => return None,
    };
    if !line
        .get(marker_end)
        .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
    {
        return None;
    }
    let padding = line[marker_end..]
        .iter()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    let consumed = if padding <= 4 { padding } else { 1 };
    let end = marker_end + consumed;
    Some((&line[end..], indentation_columns(&line[..end])))
}

fn ordered_marker_end(line: &[u8], start: usize) -> Option<usize> {
    let digits = line[start..]
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if !(1..=9).contains(&digits) {
        return None;
    }
    matches!(line.get(start + digits), Some(b'.' | b')')).then_some(start + digits + 1)
}

fn strip_indentation(line: &[u8], expected: usize) -> Option<&[u8]> {
    let mut columns = 0;
    for (index, byte) in line.iter().enumerate() {
        columns = match byte {
            b' ' => columns + 1,
            b'\t' => columns + 4 - (columns % 4),
            _ => return None,
        };
        if columns >= expected {
            return Some(&line[index + 1..]);
        }
    }
    None
}

fn indentation_columns(line: &[u8]) -> usize {
    line.iter().fold(0, |columns, byte| match byte {
        b'\t' => columns + 4 - (columns % 4),
        _ => columns + 1,
    })
}

#[cfg(test)]
#[path = "container/tests.rs"]
mod tests;
