use std::borrow::Cow;

#[derive(Clone, Copy)]
enum ContainerStep {
    Blockquote,
    ListIndent(usize),
}

#[derive(Clone)]
pub(super) struct ContainerPrefix {
    steps: Vec<ContainerStep>,
    can_interrupt_paragraph: bool,
}

impl ContainerPrefix {
    pub(super) fn from_opening_line(mut line: &[u8]) -> (&[u8], Self) {
        let mut steps = Vec::new();
        let mut can_interrupt_paragraph = true;
        loop {
            if let Some((remainder, _)) = strip_one_blockquote(line) {
                steps.push(ContainerStep::Blockquote);
                line = remainder;
            } else if let Some((remainder, indent, can_interrupt)) = strip_one_list_marker(line) {
                steps.push(ContainerStep::ListIndent(indent));
                can_interrupt_paragraph &= can_interrupt;
                line = remainder;
            } else {
                break;
            }
        }
        (
            line,
            Self {
                steps,
                can_interrupt_paragraph,
            },
        )
    }

    pub(super) fn can_interrupt_paragraph(&self) -> bool {
        self.can_interrupt_paragraph
    }

    pub(super) fn strip_line<'line>(&self, line: &'line [u8]) -> Option<Cow<'line, [u8]>> {
        if line.iter().all(|byte| matches!(byte, b' ' | b'\t')) {
            return Some(Cow::Borrowed(&line[line.len()..]));
        }
        let mut line = Cow::Borrowed(line);
        for step in &self.steps {
            let (consumed, residual_spaces) = match step {
                ContainerStep::Blockquote => {
                    let (remainder, residual_spaces) = strip_one_blockquote(&line)?;
                    (line.len() - remainder.len(), residual_spaces)
                }
                ContainerStep::ListIndent(indent) => indentation_prefix(&line, *indent)?,
            };
            line = strip_prefix(line, consumed, residual_spaces);
        }
        Some(line)
    }
}

fn strip_one_blockquote(line: &[u8]) -> Option<(&[u8], usize)> {
    let spaces = line.iter().take_while(|byte| **byte == b' ').count();
    if spaces > 3 || line.get(spaces) != Some(&b'>') {
        return None;
    }
    let marker_end = spaces + 1;
    let (end, residual_spaces) = match line.get(marker_end) {
        Some(b' ') => (marker_end + 1, 0),
        Some(b'\t') => {
            let end = marker_end + 1;
            (end, indentation_columns(&line[..end]) - (marker_end + 1))
        }
        _ => (marker_end, 0),
    };
    Some((&line[end..], residual_spaces))
}

fn strip_one_list_marker(line: &[u8]) -> Option<(&[u8], usize, bool)> {
    let leading = line.iter().take_while(|byte| **byte == b' ').count();
    if leading > 3 {
        return None;
    }
    let (marker_end, can_interrupt) = match line.get(leading)? {
        b'-' | b'+' | b'*' => (leading + 1, true),
        byte if byte.is_ascii_digit() => ordered_marker_end(line, leading)?,
        _ => return None,
    };
    if !line
        .get(marker_end)
        .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
    {
        return None;
    }
    let padding_bytes = line[marker_end..]
        .iter()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    let padding_end = marker_end + padding_bytes;
    let padding_columns =
        indentation_columns(&line[..padding_end]) - indentation_columns(&line[..marker_end]);
    let consumed = match padding_columns {
        0..=4 => padding_bytes,
        _ => 1,
    };
    let end = marker_end + consumed;
    Some((
        &line[end..],
        indentation_columns(&line[..end]),
        can_interrupt,
    ))
}

fn ordered_marker_end(line: &[u8], start: usize) -> Option<(usize, bool)> {
    let digits = line[start..]
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if !(1..=9).contains(&digits) {
        return None;
    }
    matches!(line.get(start + digits), Some(b'.' | b')')).then(|| {
        let starts_at_one = line[start..start + digits]
            .iter()
            .fold(0_u64, |value, digit| value * 10 + u64::from(digit - b'0'))
            == 1;
        (start + digits + 1, starts_at_one)
    })
}

fn indentation_prefix(line: &[u8], expected: usize) -> Option<(usize, usize)> {
    let mut columns = 0;
    for (index, byte) in line.iter().enumerate() {
        columns = match byte {
            b' ' => columns + 1,
            b'\t' => columns + 4 - (columns % 4),
            _ => return None,
        };
        if columns >= expected {
            return Some((index + 1, columns - expected));
        }
    }
    None
}

fn strip_prefix<'line>(
    line: Cow<'line, [u8]>,
    consumed: usize,
    residual_spaces: usize,
) -> Cow<'line, [u8]> {
    if residual_spaces == 0 {
        return match line {
            Cow::Borrowed(line) => Cow::Borrowed(&line[consumed..]),
            Cow::Owned(mut line) => {
                line.drain(..consumed);
                Cow::Owned(line)
            }
        };
    }
    let mut normalized = Vec::with_capacity(residual_spaces + line.len() - consumed);
    normalized.resize(residual_spaces, b' ');
    normalized.extend_from_slice(&line[consumed..]);
    Cow::Owned(normalized)
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
