use super::heredoc;
use super::{flush_command, flush_word, push_char, take_expansion, Command, Parse};

struct Scan<'a> {
    chars: &'a [char],
    index: usize,
    line: usize,
    word: String,
    word_line: usize,
    quoting: Option<char>,
    current: Command,
    commands: Vec<Command>,
    pending: Option<heredoc::Opener>,
    unterminated: bool,
    stop: bool,
}

pub(super) fn run(script: &str) -> Parse {
    let chars: Vec<char> = script.chars().collect();
    let mut scan = Scan {
        chars: &chars,
        index: 0,
        line: 1,
        word: String::new(),
        word_line: 1,
        quoting: None,
        current: Command {
            dynamic: false,
            words: Vec::new(),
        },
        commands: Vec::new(),
        pending: None,
        unterminated: false,
        stop: false,
    };
    while scan.index < scan.chars.len() && !scan.step() {}
    flush_word(&mut scan.word, &mut scan.current, scan.word_line);
    flush_command(&mut scan.current, &mut scan.commands);
    Parse {
        commands: scan.commands,
        unterminated_heredoc: scan.unterminated || scan.pending.is_some(),
    }
}

impl<'a> Scan<'a> {
    fn step(&mut self) -> bool {
        let ch = self.chars[self.index];
        if self.line_continue(ch)
            || self.quoted(ch)
            || self.start_quote(ch)
            || self.expansion(ch)
            || self.heredoc_op()
        {
            return false;
        }
        if self.separator(ch) {
            return self.stop;
        }
        if self.whitespace(ch) {
            return false;
        }
        push_char(&mut self.word, &mut self.word_line, self.line, ch);
        self.index += 1;
        false
    }

    fn line_continue(&mut self, ch: char) -> bool {
        if self.quoting.is_some() || ch != '\\' || self.chars.get(self.index + 1) != Some(&'\n') {
            return false;
        }
        self.index += 2;
        self.line += 1;
        true
    }

    fn quoted(&mut self, ch: char) -> bool {
        let Some(quote) = self.quoting else {
            return false;
        };
        if ch == quote {
            self.quoting = None;
        } else {
            push_char(&mut self.word, &mut self.word_line, self.line, ch);
            if ch == '\n' {
                self.line += 1;
            }
        }
        self.index += 1;
        true
    }

    fn start_quote(&mut self, ch: char) -> bool {
        if ch != '\'' && ch != '"' {
            return false;
        }
        self.quoting = Some(ch);
        self.index += 1;
        true
    }

    fn expansion(&mut self, ch: char) -> bool {
        if ch == '`' {
            self.current.dynamic = true;
        }
        if ch != '$' || !matches!(self.chars.get(self.index + 1), Some(&'(' | &'{')) {
            return false;
        }
        self.current.dynamic = true;
        if self.word.is_empty() {
            self.word_line = self.line;
        }
        self.index = take_expansion(self.chars, self.index, &mut self.word, &mut self.line);
        true
    }

    fn heredoc_op(&mut self) -> bool {
        match heredoc::look(self.chars, self.index) {
            Some(heredoc::Look::Herestring { index }) => {
                flush_word(&mut self.word, &mut self.current, self.word_line);
                self.index = index;
                true
            }
            Some(heredoc::Look::Document { index, opener }) => {
                flush_word(&mut self.word, &mut self.current, self.word_line);
                flush_command(&mut self.current, &mut self.commands);
                self.pending = Some(opener);
                self.index = index;
                true
            }
            None => false,
        }
    }

    fn separator(&mut self, ch: char) -> bool {
        if ch != '\n'
            && ch != ';'
            && ch != '|'
            && !(ch == '&' && self.chars.get(self.index + 1) == Some(&'&'))
        {
            return false;
        }
        flush_word(&mut self.word, &mut self.current, self.word_line);
        flush_command(&mut self.current, &mut self.commands);
        if ch == '\n' {
            if let Some(opener) = self.pending.take() {
                let body = heredoc::skip_body(self.chars, self.index, self.line, &opener);
                self.index = body.index;
                self.line = body.line;
                if !body.terminated {
                    self.unterminated = true;
                    self.stop = true;
                }
                return true;
            }
            self.line += 1;
        }
        if ch == '&' || (ch == '|' && self.chars.get(self.index + 1) == Some(&'|')) {
            self.index += 1;
        }
        self.index += 1;
        true
    }

    fn whitespace(&mut self, ch: char) -> bool {
        if !ch.is_whitespace() && ch != '<' && ch != '>' {
            return false;
        }
        flush_word(&mut self.word, &mut self.current, self.word_line);
        self.index += 1;
        true
    }
}
