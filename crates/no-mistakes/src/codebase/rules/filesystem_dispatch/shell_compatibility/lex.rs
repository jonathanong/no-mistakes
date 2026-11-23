mod nested;
mod redirection;
mod word;

pub(super) use word::{assignment, identifier, Word};

pub(super) struct Lexer<'a> {
    source: &'a str,
    offset: usize,
    pub(super) line: usize,
    heredocs: Vec<(String, bool)>,
}

impl<'a> Lexer<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self {
            source,
            offset: 0,
            line: 1,
            heredocs: Vec::new(),
        }
    }

    fn current(&self) -> Option<u8> {
        self.source.as_bytes().get(self.offset).copied()
    }

    fn rest(&self) -> &'a str {
        &self.source[self.offset..]
    }

    fn advance(&mut self) {
        let ch = self
            .rest()
            .chars()
            .next()
            .expect("lexer has input to advance");
        self.line += usize::from(ch == '\n');
        self.offset += ch.len_utf8();
    }

    fn take(&mut self, token: &str) -> bool {
        if self.rest().starts_with(token) {
            self.offset += token.len();
            true
        } else {
            false
        }
    }

    fn continuation(&mut self) -> bool {
        if self.take("\\\n") || self.take("\\\r\n") {
            self.line += 1;
            true
        } else {
            false
        }
    }

    fn skip_comment(&mut self) {
        while self.current().is_some_and(|byte| byte != b'\n') {
            self.advance();
        }
    }

    pub(super) fn commands(mut self, mut visit: impl FnMut(&[Word<'a>], usize)) {
        let mut words = Vec::new();
        let mut scope = 0usize;
        while let Some(byte) = self.current() {
            if self.continuation() {
                continue;
            }
            match byte {
                b' ' | b'\t' | b'\r' => self.advance(),
                b'#' => self.skip_comment(),
                b'{' | b'}' if self.group_boundary(&words) => {
                    visit(&words, scope);
                    words.clear();
                    self.advance();
                    if byte == b'{' {
                        scope += 1;
                    } else {
                        scope = scope.saturating_sub(1);
                        visit(&[], scope);
                    }
                }
                b'\n' | b';' | b'|' | b'&' | b'(' | b')' => {
                    visit(&words, scope);
                    words.clear();
                    if self.take("((") {
                        self.skip_group(b')');
                        self.take(")");
                        continue;
                    }
                    self.advance();
                    if byte == b'\n' {
                        self.skip_heredocs();
                    }
                }
                b'<' | b'>' => {
                    if let Some(word) = self.redirection() {
                        words.push(word);
                    }
                }
                _ => words.push(self.word()),
            }
        }
        visit(&words, scope);
    }

    fn group_boundary(&self, words: &[Word<'_>]) -> bool {
        if command_words(words)
            .next()
            .is_some_and(|word| word.raw != "function")
        {
            return false;
        }
        self.source
            .as_bytes()
            .get(self.offset + 1)
            .is_none_or(|byte| {
                byte.is_ascii_whitespace() || matches!(byte, b';' | b'&' | b'|' | b')')
            })
    }
}

pub(super) fn command_words<'w, 'a>(words: &'w [Word<'a>]) -> impl Iterator<Item = &'w Word<'a>> {
    words
        .iter()
        .filter(|word| !word.redirect)
        .skip_while(|word| {
            matches!(
                word.raw,
                "!" | "if" | "then" | "elif" | "else" | "while" | "until" | "do"
            )
        })
}
