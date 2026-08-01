#[derive(Clone, Copy)]
enum ContainerStep {
    Blockquote,
    ListIndent(usize),
}

#[derive(Clone, Default)]
pub(super) struct ContainerPrefix {
    steps: Vec<ContainerStep>,
}

impl ContainerPrefix {
    pub(super) fn from_opening_line(mut line: &[u8]) -> (&[u8], Self) {
        let mut steps = Vec::new();
        loop {
            if let Some(remainder) = strip_one_blockquote(line) {
                steps.push(ContainerStep::Blockquote);
                line = remainder;
            } else if let Some((remainder, indent)) = strip_one_list_marker(line) {
                steps.push(ContainerStep::ListIndent(indent));
                line = remainder;
            } else {
                break;
            }
        }
        (line, Self { steps })
    }

    pub(super) fn strip_line<'line>(&self, mut line: &'line [u8]) -> Option<&'line [u8]> {
        if line.iter().all(|byte| byte.is_ascii_whitespace()) {
            return Some(&line[line.len()..]);
        }
        for step in &self.steps {
            line = match step {
                ContainerStep::Blockquote => strip_one_blockquote(line)?,
                ContainerStep::ListIndent(indent) => strip_indentation(line, *indent)?,
            };
        }
        Some(line)
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
