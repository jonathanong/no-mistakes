use super::{Command, Word};

pub(super) fn commands(script: &str) -> Vec<Command> {
    let chars: Vec<char> = script.chars().collect();
    Tok::new(&chars).finish()
}

mod group;
mod heredoc;
mod quote;

struct Tok<'a> {
    chars: &'a [char],
    index: usize,
    line: usize,
    word: String,
    word_line: usize,
    open: bool,
    word_dynamic: bool,
    quote: Option<char>,
    command: Command,
    commands: Vec<Command>,
    pending_heredocs: Vec<(String, bool)>,
}

impl<'a> Tok<'a> {
    fn new(chars: &'a [char]) -> Self {
        Self {
            chars,
            index: 0,
            line: 1,
            word: String::new(),
            word_line: 1,
            open: false,
            word_dynamic: false,
            quote: None,
            command: Command::default(),
            commands: Vec::new(),
            pending_heredocs: Vec::new(),
        }
    }

    fn finish(mut self) -> Vec<Command> {
        while self.index < self.chars.len() {
            self.step();
        }
        if self.quote.is_some() {
            self.command.dynamic = true;
        }
        self.flush_word();
        self.flush_command();
        self.commands
    }

    fn step(&mut self) -> bool {
        self.quoted_single()
            || self.quoted_double()
            || self.line_continue()
            || self.escaped()
            || self.start_quote()
            || self.capture_backtick()
            || self.capture_dollar()
            || self.capture_process()
            || self.skip_heredoc()
            || self.comment()
            || self.separator()
            || self.push_plain()
    }

    fn ch(&self) -> char {
        self.chars[self.index]
    }

    fn line_continue(&mut self) -> bool {
        if self.ch() == '\\' && self.chars.get(self.index + 1) == Some(&'\n') {
            self.line += 1;
            self.index += 2;
            return true;
        }
        false
    }

    fn escaped(&mut self) -> bool {
        if self.ch() == '\\' && self.index + 1 < self.chars.len() {
            self.mark_open();
            self.word.push(self.chars[self.index + 1]);
            self.index += 2;
            return true;
        }
        false
    }

    fn start_quote(&mut self) -> bool {
        let ch = self.ch();
        if ch == '\'' || ch == '"' {
            self.mark_open();
            self.quote = Some(ch);
            self.index += 1;
            return true;
        }
        false
    }

    fn comment(&mut self) -> bool {
        if self.ch() == '#' && !self.open {
            while self.index < self.chars.len() && self.chars[self.index] != '\n' {
                self.index += 1;
            }
            return true;
        }
        false
    }

    fn separator(&mut self) -> bool {
        let ch = self.ch();
        if !(ch == '\n' || ch == ';' || ch == '|' || ch == '&' || ch.is_whitespace()) {
            return false;
        }
        self.flush_word();
        if ch == '\n' {
            self.line += 1;
            self.flush_command();
            self.index += 1;
            self.skip_pending_bodies();
            return true;
        } else if ch == ';' || ch == '|' || ch == '&' {
            if (ch == '|' || ch == '&') && self.chars.get(self.index + 1) == Some(&ch) {
                self.index += 1;
            }
            self.flush_command();
        }
        self.index += 1;
        true
    }

    fn push_plain(&mut self) -> bool {
        self.mark_open();
        self.word.push(self.ch());
        self.index += 1;
        true
    }

    fn mark_open(&mut self) {
        if !self.open {
            self.word_line = self.line;
            self.open = true;
        }
    }

    fn flush_word(&mut self) {
        if self.open {
            self.command.words.push(Word {
                text: std::mem::take(&mut self.word),
                line: self.word_line,
                dynamic: self.word_dynamic,
            });
            self.word_dynamic = false;
            self.open = false;
        }
    }

    fn flush_command(&mut self) {
        if self.command.dynamic || !self.command.words.is_empty() {
            self.commands.push(std::mem::take(&mut self.command));
        }
    }
}
