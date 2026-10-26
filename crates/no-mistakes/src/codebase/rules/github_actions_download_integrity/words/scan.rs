use super::heredoc;
use super::{flush_command, push_char, skip_arithmetic, Command, Parse};

mod word;

struct Scan<'a> {
    chars: &'a [char],
    index: usize,
    line: usize,
    word: String,
    word_line: usize,
    word_expansion_may_split: bool,
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
        word_expansion_may_split: false,
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
    scan.flush_word();
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
            || self.comment(ch)
            || self.start_quote(ch)
            || self.expansion(ch)
            || self.arithmetic()
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

    /// `#` starts a comment only at the beginning of a word, outside quotes.
    /// The newline stays for `separator`, so a here-document opener on the
    /// same line still skips its body (`cat <<EOF # note`).
    fn comment(&mut self, ch: char) -> bool {
        if ch != '#' || self.quoting.is_some() || !self.word.is_empty() {
            return false;
        }
        while self.index < self.chars.len() && self.chars[self.index] != '\n' {
            self.index += 1;
        }
        true
    }

    /// `(( ... ))` is an arithmetic command, not a here-document. A shift
    /// inside it must not swallow the rest of the step.
    fn arithmetic(&mut self) -> bool {
        if self.quoting.is_some()
            || !self.word.is_empty()
            || !self.current.words.is_empty()
            || self.chars.get(self.index) != Some(&'(')
            || self.chars.get(self.index + 1) != Some(&'(')
        {
            return false;
        }
        self.index = skip_arithmetic(self.chars, self.index, &mut self.line);
        true
    }

    fn heredoc_op(&mut self) -> bool {
        match heredoc::look(self.chars, self.index) {
            Some(heredoc::Look::Herestring { index }) => {
                self.flush_word();
                self.index = index;
                true
            }
            Some(heredoc::Look::Document { index, opener }) => {
                self.flush_word();
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
        self.flush_word();
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
        self.flush_word();
        self.index += 1;
        true
    }
}
