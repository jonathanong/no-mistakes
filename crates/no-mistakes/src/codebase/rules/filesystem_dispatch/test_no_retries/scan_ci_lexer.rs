#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Sep {
    #[default]
    Start,
    Or,
    Other,
}

pub(super) struct Segment {
    pub(super) words: Vec<String>,
    pub(super) line: usize,
    pub(super) preceded_by: Sep,
}

#[derive(Default)]
struct Lexer {
    out: Vec<Segment>,
    words: Vec<String>,
    word: String,
    line: usize,
    seg_line: usize,
    next_sep: Sep,
    quote: Option<char>,
}

impl Lexer {
    fn flush_word(&mut self) {
        if !self.word.is_empty() {
            self.words.push(std::mem::take(&mut self.word));
        }
    }

    fn flush_segment(&mut self, sep: Sep) {
        self.flush_word();
        let emitted = !self.words.is_empty();
        if emitted {
            self.out.push(Segment {
                words: std::mem::take(&mut self.words),
                line: self.seg_line,
                preceded_by: self.next_sep,
            });
        }
        if emitted || sep == Sep::Or {
            self.next_sep = sep;
        }
    }

    fn push(&mut self, ch: char) {
        if self.words.is_empty() && self.word.is_empty() {
            self.seg_line = self.line;
        }
        self.word.push(ch);
    }

    fn escape(&mut self, chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
        match chars.next() {
            Some('\n') => self.line += 1,
            Some(esc) => self.push(esc),
            None => {}
        }
    }

    fn step_quoted(
        &mut self,
        q: char,
        ch: char,
        chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    ) {
        if ch == '\n' {
            self.line += 1;
        }
        if ch == q {
            self.quote = None;
        } else if q == '"' && ch == '\\' {
            self.escape(chars);
        } else {
            self.push(ch);
        }
    }
}

pub(super) fn split_shell_segments(script: &str) -> Vec<Segment> {
    let mut lex = Lexer {
        line: 1,
        seg_line: 1,
        ..Lexer::default()
    };
    let mut chars = script.chars().peekable();
    while let Some(ch) = chars.next() {
        if let Some(q) = lex.quote {
            lex.step_quoted(q, ch, &mut chars);
            continue;
        }
        match ch {
            '\\' => lex.escape(&mut chars),
            '\'' | '"' => lex.quote = Some(ch),
            '#' if lex.word.is_empty() => {
                while chars.peek().is_some_and(|c| *c != '\n') {
                    chars.next();
                }
            }
            '\n' | ';' | '|' | '&' => {
                let doubled = matches!(ch, '|' | '&') && chars.peek() == Some(&ch);
                if doubled {
                    chars.next();
                }
                lex.flush_segment(if ch == '|' && doubled {
                    Sep::Or
                } else {
                    Sep::Other
                });
                if ch == '\n' {
                    lex.line += 1;
                }
            }
            c if c.is_whitespace() => lex.flush_word(),
            c => lex.push(c),
        }
    }
    lex.flush_segment(Sep::Other);
    lex.out
}
