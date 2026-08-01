#[derive(Default)]
pub(super) struct CodeSpanScanner {
    delimiter: Option<usize>,
}

impl CodeSpanScanner {
    pub(super) fn is_masking_markdown(&self) -> bool {
        self.delimiter.is_some()
    }

    pub(super) fn observe_source(&mut self, source: &[u8], start: usize, end: usize) {
        let mut index = start;
        while index < end {
            if source[index] != b'`' || is_escaped(source, index) {
                index += 1;
                continue;
            }
            let length = backtick_run(source, index);
            match self.delimiter {
                Some(delimiter) if length == delimiter => self.delimiter = None,
                Some(_) => {}
                None if has_closer_before_blank(source, index + length, length) => {
                    self.delimiter = Some(length);
                }
                None => {}
            }
            index += length;
        }
    }
}

fn has_closer_before_blank(source: &[u8], mut index: usize, delimiter: usize) -> bool {
    let mut line_has_content = true;
    while index < source.len() {
        match source[index] {
            b'`' if !is_escaped(source, index) => {
                let length = backtick_run(source, index);
                if length == delimiter {
                    return true;
                }
                line_has_content = true;
                index += length;
            }
            b'\n' | b'\r' => {
                if !line_has_content {
                    return false;
                }
                if source[index] == b'\r' && source.get(index + 1) == Some(&b'\n') {
                    index += 1;
                }
                line_has_content = false;
                index += 1;
            }
            b' ' | b'\t' => index += 1,
            _ => {
                line_has_content = true;
                index += 1;
            }
        }
    }
    false
}

fn backtick_run(source: &[u8], start: usize) -> usize {
    source[start..]
        .iter()
        .take_while(|byte| **byte == b'`')
        .count()
}

fn is_escaped(source: &[u8], index: usize) -> bool {
    source[..index]
        .iter()
        .rev()
        .take_while(|byte| **byte == b'\\')
        .count()
        % 2
        == 1
}

#[cfg(test)]
#[path = "code_span/tests.rs"]
mod tests;
