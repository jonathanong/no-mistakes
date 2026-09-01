pub(super) struct LexState {
    pub(super) quote: Option<u8>,
    escaped: bool,
    pub(super) regex: bool,
    regex_class: bool,
    pub(super) regex_allowed: bool,
    pub(super) line_comment: bool,
    pub(super) block_comment: bool,
    non_code_start: Option<usize>,
}

impl LexState {
    pub(super) fn new() -> Self {
        Self {
            quote: None,
            escaped: false,
            regex: false,
            regex_class: false,
            regex_allowed: true,
            line_comment: false,
            block_comment: false,
            non_code_start: None,
        }
    }

    pub(super) fn enter_non_code(&mut self, start: usize) {
        self.non_code_start = Some(start);
    }

    pub(super) fn finish_non_code(&mut self, end: usize, ranges: &mut Vec<(usize, usize)>) {
        if let Some(start) = self.non_code_start.take() {
            ranges.push((start, end));
        }
    }

    pub(super) fn skip_non_code(
        &mut self,
        bytes: &[u8],
        index: usize,
        ranges: &mut Vec<(usize, usize)>,
    ) -> Option<usize> {
        let byte = bytes[index];
        if self.line_comment {
            self.line_comment = byte != b'\n';
            if !self.line_comment {
                self.finish_non_code(index, ranges);
            }
            return Some(index + 1);
        }
        if self.block_comment {
            if byte == b'*' && bytes.get(index + 1) == Some(&b'/') {
                self.block_comment = false;
                self.finish_non_code(index + 2, ranges);
                return Some(index + 2);
            }
            return Some(index + 1);
        }
        if let Some(delimiter) = self.quote {
            if self.escaped {
                if byte != b'\r' || bytes.get(index + 1) != Some(&b'\n') {
                    self.escaped = false;
                }
            } else if byte == b'\\' {
                self.escaped = true;
            } else if delimiter != b'`' && matches!(byte, b'\n' | b'\r') {
                self.quote = None;
                self.finish_non_code(index, ranges);
            } else if byte == delimiter {
                self.quote = None;
                self.regex_allowed = false;
                self.finish_non_code(index + 1, ranges);
            }
            return Some(index + 1);
        }
        if self.regex {
            if self.escaped {
                self.escaped = false;
            } else {
                match byte {
                    b'\\' => self.escaped = true,
                    b'[' => self.regex_class = true,
                    b']' => self.regex_class = false,
                    b'/' if !self.regex_class => {
                        self.regex = false;
                        self.regex_allowed = false;
                        self.finish_non_code(index + 1, ranges);
                    }
                    b'\n' => {
                        self.regex = false;
                        self.regex_class = false;
                        self.finish_non_code(index, ranges);
                    }
                    _ => {}
                }
            }
            return Some(index + 1);
        }
        None
    }
}
