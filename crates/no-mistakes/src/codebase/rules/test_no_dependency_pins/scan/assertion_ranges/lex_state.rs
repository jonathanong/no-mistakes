pub(super) struct LexState {
    pub(super) quote: Option<u8>,
    escaped: bool,
    pub(super) regex: bool,
    regex_class: bool,
    pub(super) regex_allowed: bool,
    pub(super) line_comment: bool,
    pub(super) block_comment: bool,
    non_code_start: Option<usize>,
    control_parens: Vec<bool>,
    template_braces: Vec<usize>,
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
            control_parens: Vec::new(),
            template_braces: Vec::new(),
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

    pub(super) fn open_brace(&mut self) {
        if let Some(depth) = self.template_braces.last_mut() {
            *depth += 1;
        }
    }

    pub(super) fn open_paren(&mut self, control_condition: bool) {
        self.control_parens.push(control_condition);
    }

    pub(super) fn close_paren(&mut self) -> bool {
        self.control_parens.pop().unwrap_or(false)
    }

    pub(super) fn close_brace(&mut self, index: usize) -> bool {
        let Some(depth) = self.template_braces.last_mut() else {
            return false;
        };
        *depth -= 1;
        if *depth > 0 {
            return false;
        }
        self.template_braces.pop();
        self.quote = Some(b'`');
        self.regex_allowed = false;
        self.enter_non_code(index + 1);
        true
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
            } else if delimiter == b'`' && byte == b'$' && bytes.get(index + 1) == Some(&b'{') {
                self.quote = None;
                self.finish_non_code(index, ranges);
                self.template_braces.push(1);
                self.regex_allowed = true;
                return Some(index + 2);
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

pub(super) fn follows_control_condition(
    bytes: &[u8],
    open_paren: usize,
    non_code_ranges: &[(usize, usize)],
) -> bool {
    let end = super::skip_trivia(bytes, open_paren, non_code_ranges);
    if [b"if".as_slice(), b"while", b"for", b"with"]
        .iter()
        .any(|token| standalone_token_ends_at(bytes, end, token))
    {
        return true;
    }
    let Some(await_start) = end.checked_sub(b"await".len()) else {
        return false;
    };
    standalone_token_ends_at(bytes, end, b"await")
        && standalone_token_ends_at(
            bytes,
            super::skip_trivia(bytes, await_start, non_code_ranges),
            b"for",
        )
}

fn standalone_token_ends_at(bytes: &[u8], end: usize, token: &[u8]) -> bool {
    let Some(start) = end.checked_sub(token.len()) else {
        return false;
    };
    &bytes[start..end] == token
        && (start == 0
            || !(bytes[start - 1].is_ascii_alphanumeric()
                || matches!(bytes[start - 1], b'_' | b'$' | b'.' | b'#')))
}
