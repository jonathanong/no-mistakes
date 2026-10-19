use super::{Command, Word};

pub(super) fn commands(script: &str) -> Vec<Command> {
    let chars: Vec<char> = script.chars().collect();
    Tok::new(&chars).finish()
}

mod quote;

struct Tok<'a> {
    chars: &'a [char],
    index: usize,
    line: usize,
    word: String,
    word_line: usize,
    open: bool,
    quote: Option<char>,
    subst: i32,
    backtick: bool,
    command: Command,
    commands: Vec<Command>,
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
            quote: None,
            subst: 0,
            backtick: false,
            command: Command::default(),
            commands: Vec::new(),
        }
    }

    fn finish(mut self) -> Vec<Command> {
        while self.index < self.chars.len() {
            self.step();
        }
        if self.quote.is_some() || self.backtick || self.subst > 0 {
            self.command.dynamic = true;
        }
        self.flush_word();
        self.flush_command();
        self.commands
    }

    fn step(&mut self) -> bool {
        self.quoted_single()
            || self.inside_backtick()
            || self.inside_subst()
            || self.quoted_double()
            || self.line_continue()
            || self.escaped()
            || self.start_quote()
            || self.start_backtick()
            || self.start_subst()
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

    fn start_backtick(&mut self) -> bool {
        if self.ch() == '`' {
            self.command.dynamic = true;
            self.backtick = true;
            self.index += 1;
            return true;
        }
        false
    }

    fn start_subst(&mut self) -> bool {
        if self.ch() == '$' && self.chars.get(self.index + 1) == Some(&'(') {
            self.command.dynamic = true;
            self.subst = 1;
            self.index += 2;
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
            });
            self.open = false;
        }
    }

    fn flush_command(&mut self) {
        if self.command.dynamic || !self.command.words.is_empty() {
            self.commands.push(std::mem::take(&mut self.command));
        }
    }
}
