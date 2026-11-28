use super::{command_words, function_keyword, Boundary, Cases, Function, Lexer, Word};

impl<'a> Lexer<'a> {
    pub(super) fn case_punctuation(
        &mut self,
        byte: u8,
        cases: &mut Cases,
        words: &mut Vec<Word<'a>>,
    ) -> bool {
        let pattern_ends = cases.punctuation(byte, words.is_empty());
        if pattern_ends {
            self.advance();
            return true;
        }
        words.push(self.separator_word(byte));
        false
    }

    pub(super) fn arithmetic_word(&mut self, allow_for_header: bool) -> Option<Word<'a>> {
        if self.arith_unclosed {
            return None;
        }
        let start = self.offset;
        let line = self.line;
        self.take("((");
        let closed = self.skip_group(b')');
        if !closed {
            self.arith_unclosed = true;
            self.offset = start;
            self.line = line;
            return None;
        }
        if self
            .source
            .get(start + 2..self.offset.saturating_sub(1))
            .is_some_and(|body| allow_for_header || !body.contains([';', '\n']))
            && self.take(")")
        {
            Some(Word::new(&self.source[start..self.offset], line))
        } else {
            self.offset = start;
            self.line = line;
            None
        }
    }

    pub(super) fn push_word(
        &mut self,
        words: &mut Vec<Word<'a>>,
        conditional: &mut bool,
        cases: &mut Cases,
        is_bash: bool,
    ) -> bool {
        let word = self.word();
        self.finish_pushed_word(word, words, conditional, cases, is_bash)
    }

    pub(super) fn finish_pushed_word(
        &mut self,
        mut word: Word<'a>,
        words: &mut Vec<Word<'a>>,
        conditional: &mut bool,
        cases: &mut Cases,
        is_bash: bool,
    ) -> bool {
        word.redirect = !*conditional && self.is_fd_prefix(word.raw);
        if word.raw == "[[" && !cases.in_pattern() && command_words(words).next().is_none() {
            *conditional = true;
        } else if word.raw == "]]" {
            *conditional = false;
        }
        let starts_patterns = is_bash && !word.redirect && cases.word(word.raw, words);
        words.push(word);
        starts_patterns
    }

    pub(super) fn command_separator(
        &mut self,
        byte: u8,
        words: &[Word<'a>],
        pending_function: &mut Option<Function<'a>>,
        cases: &mut Cases,
        is_bash: bool,
    ) -> Boundary<'a> {
        *pending_function = if matches!(byte, b'\n' | b';') {
            pending_function.or_else(|| function_keyword(words))
        } else {
            None
        };
        let boundary = self.separator(byte);
        if is_bash && matches!(boundary, Boundary::CaseArm(_)) {
            cases.next_arm();
        }
        if byte == b'\n' {
            self.skip_heredocs();
        }
        boundary
    }

    pub(super) fn separator(&mut self, byte: u8) -> Boundary<'a> {
        use super::CaseArm;
        use Boundary::{And, End, Or, Other};
        match byte {
            b'&' if self.take("&&") => And,
            b'|' if self.take("||") => Or,
            b';' if self.take(";;&") => Boundary::CaseArm(CaseArm::TestNext),
            b';' if self.take(";;") => Boundary::CaseArm(CaseArm::Break),
            b';' if self.take(";&") => Boundary::CaseArm(CaseArm::Fallthrough),
            _ => {
                self.advance();
                if matches!(byte, b'\n' | b';' | b')') {
                    End
                } else {
                    Other
                }
            }
        }
    }

    pub(super) fn separator_word(&mut self, byte: u8) -> Word<'a> {
        let start = self.offset;
        let line = self.line;
        self.separator(byte);
        Word::new(&self.source[start..self.offset], line)
    }

    pub(super) fn completes_list(&self) -> bool {
        matches!(
            self.rest()
                .trim_start_matches([' ', '\t', '\r'])
                .as_bytes()
                .first(),
            None | Some(b'\n' | b';' | b'#' | b')' | b'}')
        )
    }

    pub(super) fn brace_boundary(&self) -> bool {
        self.source
            .as_bytes()
            .get(self.offset + 1)
            .is_none_or(|byte| {
                byte.is_ascii_whitespace()
                    || matches!(byte, b';' | b'&' | b'|' | b')' | b'<' | b'>')
            })
    }
}
