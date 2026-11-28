use crate::codebase::command_lex::heredoc::Document;
use super::{flush_command, push_char, skip_arithmetic, Command, HeredocBody, Parse, Range};

mod comment;
mod redirection;
mod word;

struct Scan<'a> {
    chars: &'a [char],
    strict: bool,
    index: usize,
    line: usize,
    word: String,
    word_line: usize,
    word_quoted: bool,
    redirect_operand: bool,
    word_expansion_may_split: bool,
    word_quoted_expansion: Option<(usize, usize)>,
    word_bare_expansion: Option<(usize, usize)>,
    quoting: Option<char>,
    current: Command,
    commands: Vec<Command>,
    pending: Vec<(Document, bool)>,
    heredoc_ranges: Vec<Range<usize>>,
    heredoc_bodies: Vec<HeredocBody>,
    unterminated: bool,
    stop: bool,
    collect_skips: bool,
    skip_lines: Vec<usize>,
    in_backticks: bool,
    backtick_start: Option<(usize, usize)>,
    unterminated_substitution: bool,
    substitutions: Vec<(usize, usize, String)>,
}

pub(super) fn run(script: &str, strict: bool, collect_skips: bool) -> Parse {
    let chars: Vec<char> = script.chars().collect();
    let mut scan = Scan {
        chars: &chars,
        strict,
        index: 0,
        line: 1,
        word: String::new(),
        word_line: 1,
        word_quoted: false,
        redirect_operand: false,
        word_expansion_may_split: false,
        word_quoted_expansion: None,
        word_bare_expansion: None,
        quoting: None,
        current: Command {
            anchor: 0,
            dynamic: false,
            words: Vec::new(),
        },
        commands: Vec::new(),
        pending: Vec::new(),
        heredoc_ranges: Vec::new(),
        heredoc_bodies: Vec::new(),
        unterminated: false,
        stop: false,
        collect_skips,
        skip_lines: Vec::new(),
        in_backticks: false,
        backtick_start: None,
        unterminated_substitution: false,
        substitutions: Vec::new(),
    };
    while scan.index < scan.chars.len() && !scan.step() {}
    scan.flush_word();
    flush_command(&mut scan.current, &mut scan.commands);
    Parse {
        commands: scan.commands,
        unterminated_heredoc: scan.unterminated || !scan.pending.is_empty(),
        unterminated_quote: scan.quoting.is_some(),
        unterminated_substitution: scan.unterminated_substitution || scan.in_backticks,
        substitutions: scan.substitutions,
        skip_lines: scan.skip_lines,
        heredoc_ranges: scan.heredoc_ranges,
        heredoc_bodies: scan.heredoc_bodies,
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
            || self.redirection()
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

    fn separator(&mut self, ch: char) -> bool {
        let double_ampersand = ch == '&' && self.chars.get(self.index + 1) == Some(&'&');
        let background = self.strict
            && ch == '&'
            && self
                .index
                .checked_sub(1)
                .is_none_or(|prev| !matches!(self.chars[prev], '>' | '<') || self.escaped_at(prev))
            && self.chars.get(self.index + 1) != Some(&'>')
            && !self.escaped();
        if !matches!(ch, '\n' | ';' | '|') && !double_ampersand && !background {
            return false;
        }
        self.flush_word();
        self.redirect_operand = false;
        flush_command(&mut self.current, &mut self.commands);
        self.current.anchor = self.index + 1;
        if ch == '\n' {
            if !self.pending.is_empty() {
                self.skip_heredocs();
                return true;
            }
            self.line += 1;
        }
        if double_ampersand || (ch == '|' && self.chars.get(self.index + 1) == Some(&'|')) {
            self.index += 1;
        }
        self.index += 1;
        true
    }

    fn whitespace(&mut self, ch: char) -> bool {
        if !ch.is_whitespace() {
            return false;
        }
        self.flush_word();
        self.index += 1;
        true
    }
}
