struct LexState {
    quote: Option<u8>,
    escaped: bool,
    regex: bool,
    regex_class: bool,
    regex_allowed: bool,
    line_comment: bool,
    block_comment: bool,
}

impl LexState {
    fn new() -> Self {
        Self {
            quote: None,
            escaped: false,
            regex: false,
            regex_class: false,
            regex_allowed: true,
            line_comment: false,
            block_comment: false,
        }
    }

    fn skip_non_code(&mut self, bytes: &[u8], index: usize) -> Option<usize> {
        let byte = bytes[index];
        if self.line_comment {
            self.line_comment = byte != b'\n';
            return Some(index + 1);
        }
        if self.block_comment {
            if byte == b'*' && bytes.get(index + 1) == Some(&b'/') {
                self.block_comment = false;
                return Some(index + 2);
            }
            return Some(index + 1);
        }
        if let Some(delimiter) = self.quote {
            if self.escaped {
                self.escaped = false;
            } else if byte == b'\\' {
                self.escaped = true;
            } else if byte == delimiter {
                self.quote = None;
                self.regex_allowed = false;
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
                    }
                    b'\n' => {
                        self.regex = false;
                        self.regex_class = false;
                    }
                    _ => {}
                }
            }
            return Some(index + 1);
        }
        None
    }
}

pub(super) fn assertion_ranges(content: &str) -> Vec<(usize, usize)> {
    let bytes = content.as_bytes();
    let mut stack = Vec::new();
    let mut ranges = Vec::new();
    let mut lex = LexState::new();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        let next = bytes.get(index + 1).copied();
        if let Some(next_index) = lex.skip_non_code(bytes, index) {
            index = next_index;
            continue;
        }
        match byte {
            b'/' if next == Some(b'/') => {
                lex.line_comment = true;
                index += 2;
                continue;
            }
            b'/' if next == Some(b'*') => {
                lex.block_comment = true;
                index += 2;
                continue;
            }
            b'/' if lex.regex_allowed => lex.regex = true,
            b'/' => lex.regex_allowed = true,
            b'\'' | b'"' | b'`' => lex.quote = Some(byte),
            b'(' => {
                stack.push(expect_token_start(bytes, index));
                lex.regex_allowed = true;
            }
            b')' => {
                if let Some(Some(start)) = stack.pop() {
                    ranges.push((start, index));
                }
                lex.regex_allowed = false;
            }
            b'[' | b'{' | b',' | b';' | b':' | b'=' | b'!' | b'?' | b'&' | b'|' | b'+' | b'-'
            | b'*' | b'%' | b'~' | b'^' | b'<' | b'>' => lex.regex_allowed = true,
            b']' | b'}' | b'.' => lex.regex_allowed = false,
            byte if byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') => {
                let start = index;
                index += 1;
                while index < bytes.len()
                    && (bytes[index].is_ascii_alphanumeric() || matches!(bytes[index], b'_' | b'$'))
                {
                    index += 1;
                }
                lex.regex_allowed = matches!(
                    &bytes[start..index],
                    b"return"
                        | b"throw"
                        | b"case"
                        | b"delete"
                        | b"void"
                        | b"typeof"
                        | b"yield"
                        | b"await"
                );
                continue;
            }
            byte if !byte.is_ascii_whitespace() => lex.regex_allowed = false,
            _ => {}
        }
        index += 1;
    }
    ranges.extend(
        stack
            .into_iter()
            .flatten()
            .map(|start| (start, bytes.len())),
    );
    ranges.sort_unstable_by_key(|(start, _)| *start);
    ranges
}

pub(super) fn assertion_start(ranges: &[(usize, usize)], match_start: usize) -> usize {
    let upper = ranges.partition_point(|(start, _)| *start <= match_start);
    ranges[..upper]
        .iter()
        .rev()
        .find_map(|&(start, end)| (match_start <= end).then_some(start))
        .unwrap_or(match_start)
}

pub(super) fn expect_token_start(bytes: &[u8], open_paren: usize) -> Option<usize> {
    let mut end = open_paren;
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    for token in [b"expect.soft".as_slice(), b"expect.poll", b"expect"] {
        let Some(start) = end.checked_sub(token.len()) else {
            continue;
        };
        if &bytes[start..end] == token
            && (start == 0
                || !(bytes[start - 1].is_ascii_alphanumeric()
                    || matches!(bytes[start - 1], b'_' | b'$' | b'.')))
        {
            return Some(start);
        }
    }
    None
}
