mod heredoc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Word {
    pub(crate) text: String,
    pub(crate) line: usize,
    pub(crate) dynamic: bool,
}

struct Scan {
    out: Vec<Vec<Word>>,
    command: Vec<Word>,
    word: String,
    word_line: usize,
    word_dynamic: bool,
    in_word: bool,
    line: usize,
    single: bool,
    double: bool,
}

pub(in crate::codebase::rules) struct ScriptCommands {
    pub(crate) commands: Vec<Vec<Word>>,
    pub(crate) unterminated_heredoc: bool,
}

pub(in crate::codebase::rules) fn commands(script: &str) -> ScriptCommands {
    let (script, unterminated_heredoc) = heredoc::mask(script);
    let mut scan = Scan {
        out: Vec::new(),
        command: Vec::new(),
        word: String::new(),
        word_line: 1,
        word_dynamic: false,
        in_word: false,
        line: 1,
        single: false,
        double: false,
    };
    let mut chars = script.chars().peekable();
    while let Some(ch) = chars.next() {
        if scan.quoted_escape(ch, &mut chars) || scan.quoted(ch) || scan.comment(ch, &mut chars) {
            continue;
        }
        if ch == '\n' || ch == ';' || ch == '|' || ch == '&' {
            if ch == '\n' {
                scan.line += 1;
            }
            if (ch == '|' || ch == '&') && chars.peek() == Some(&ch) {
                chars.next();
            }
            scan.finish_word();
            scan.finish_command();
            continue;
        }
        if ch.is_whitespace() {
            scan.finish_word();
            continue;
        }
        scan.push(ch);
    }
    scan.finish_word();
    scan.finish_command();
    ScriptCommands {
        commands: scan.out,
        unterminated_heredoc,
    }
}

impl Scan {
    fn quoted_escape(
        &mut self,
        ch: char,
        chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    ) -> bool {
        if self.single || ch != '\\' {
            return false;
        }
        let Some(next) = chars.next() else {
            return true;
        };
        if next == '\n' {
            self.line += 1;
            return true;
        }
        self.push_literal(next);
        true
    }

    fn quoted(&mut self, ch: char) -> bool {
        if self.single {
            match ch {
                '\'' => self.single = false,
                '\n' => {
                    self.line += 1;
                    self.word.push(ch);
                }
                _ => self.word.push(ch),
            }
            return true;
        }
        if self.double && ch != '\\' {
            if ch == '"' {
                self.double = false;
                return true;
            }
            if ch == '\n' {
                self.line += 1;
            }
            if ch == '$' || ch == '`' {
                self.word_dynamic = true;
            }
            self.word.push(ch);
            return true;
        }
        if ch == '\'' || ch == '"' {
            self.start();
            self.single = ch == '\'';
            self.double = ch == '"';
            return true;
        }
        false
    }

    fn comment(&mut self, ch: char, chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
        if ch != '#' || self.in_word {
            return false;
        }
        while chars.peek().is_some_and(|next| *next != '\n') {
            chars.next();
        }
        true
    }

    fn push(&mut self, ch: char) {
        self.push_literal(ch);
        if ch == '$' || ch == '`' {
            self.word_dynamic = true;
        }
    }

    fn push_literal(&mut self, ch: char) {
        self.start();
        self.word.push(ch);
    }

    fn start(&mut self) {
        if !self.in_word {
            self.word_line = self.line;
            self.in_word = true;
        }
    }

    fn finish_word(&mut self) {
        if self.in_word && !self.word.is_empty() {
            let dynamic = self.word_dynamic || self.word.contains(['$', '`']);
            self.command.push(Word {
                text: std::mem::take(&mut self.word),
                line: self.word_line,
                dynamic,
            });
        }
        self.in_word = false;
        self.word_dynamic = false;
    }

    fn finish_command(&mut self) {
        if !self.command.is_empty() {
            self.out.push(std::mem::take(&mut self.command));
        }
    }
}
