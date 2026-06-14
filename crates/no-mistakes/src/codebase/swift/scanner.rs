pub(super) fn string_arg(source: &str, label: &str) -> Option<String> {
    let colon = find_label_colon(source, label)?;
    let quote = source[colon + 1..]
        .char_indices()
        .find_map(|(offset, ch)| (!ch.is_whitespace()).then_some((colon + 1 + offset, ch)))?;
    (quote.1 == '"')
        .then(|| read_quoted_string(source, quote.0).map(|(value, _)| value))
        .flatten()
}

pub(super) fn find_label_colon(source: &str, label: &str) -> Option<usize> {
    let mut scanner = Scanner::new(source);
    while let Some(index) = scanner.next_code_index() {
        let rest = &source[index..];
        if !rest.starts_with(label) {
            continue;
        }
        let before_ok = index == 0
            || !source[..index]
                .chars()
                .next_back()
                .is_some_and(is_identifier_char);
        let after = index + label.len();
        let after_ok = source[after..]
            .chars()
            .next()
            .is_some_and(|ch| !is_identifier_char(ch));
        if !before_ok || !after_ok {
            continue;
        }
        let Some((offset, ch)) = source[after..]
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace())
        else {
            continue;
        };
        if ch == ':' {
            return Some(after + offset);
        }
    }
    None
}

pub(super) fn find_matching_delimiter(
    source: &str,
    open_index: usize,
    open_char: char,
    close_char: char,
) -> Option<usize> {
    let mut scanner = Scanner::new(source);
    scanner.skip_to(open_index);
    let mut depth = 0usize;
    while let Some(index) = scanner.next_code_index() {
        let ch = source[index..].chars().next()?;
        if ch == open_char {
            depth += 1;
        } else if ch == close_char {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

pub(super) fn read_quoted_string(source: &str, quote_index: usize) -> Option<(String, usize)> {
    let mut value = String::new();
    let mut escaped = false;
    for (offset, ch) in source[quote_index + 1..].char_indices() {
        if escaped {
            value.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
            value.push(ch);
        } else if ch == '"' {
            return Some((value, quote_index + 1 + offset + ch.len_utf8()));
        } else {
            value.push(ch);
        }
    }
    None
}

pub(super) struct Scanner<'a> {
    source: &'a str,
    index: usize,
}

impl<'a> Scanner<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self { source, index: 0 }
    }

    pub(super) fn skip_to(&mut self, index: usize) {
        self.index = index;
    }

    pub(super) fn next_code_index(&mut self) -> Option<usize> {
        while self.index < self.source.len() {
            let index = self.index;
            let ch = self.source[index..].chars().next()?;
            if self.source[index..].starts_with("//") {
                self.index = self.source[index..]
                    .find('\n')
                    .map_or(self.source.len(), |offset| index + offset + 1);
                continue;
            }
            if self.source[index..].starts_with("/*") {
                self.index = self.source[index + 2..]
                    .find("*/")
                    .map_or(self.source.len(), |offset| index + 2 + offset + 2);
                continue;
            }
            if ch == '"' {
                self.index = read_quoted_string(self.source, index)
                    .map_or(self.source.len(), |(_, next)| next);
                return Some(index);
            }
            self.index += ch.len_utf8();
            return Some(index);
        }
        None
    }
}

fn is_identifier_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}
