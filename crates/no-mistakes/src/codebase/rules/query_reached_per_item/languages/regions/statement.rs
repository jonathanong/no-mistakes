use super::lex::first_word;
use super::params::parameter_names;
use super::{Frame, Header};

pub(super) struct Unbraced {
    pub(super) depth: usize,
    pub(super) delimiters: i32,
    pub(super) in_header: bool,
    pub(super) armed: bool,
}

impl Unbraced {
    pub(super) fn at(depth: usize) -> Self {
        Self {
            depth,
            delimiters: 0,
            in_header: false,
            armed: false,
        }
    }
}

pub(super) fn params_of(header: &Header, line: &str) -> Vec<String> {
    params_of_loop(matches!(header, Header::Loop(_)), line)
}

pub(super) fn params_of_loop(is_loop: bool, line: &str) -> Vec<String> {
    if is_loop {
        Vec::new()
    } else {
        parameter_names(line)
    }
}

pub(super) fn release_armed(stack: &mut Vec<Frame>, unbraced: &mut Option<Unbraced>, line: &str) {
    let Some(state) = unbraced.as_mut() else {
        return;
    };
    if !state.armed {
        return;
    }
    let word = first_word(line.trim());
    if matches!(word, "else" | "elif" | "elsif" | "elseif") {
        state.armed = false;
        state.in_header = true;
        return;
    }
    if matches!(stack.last(), Some(Frame::Loop { open, .. }) if *open == state.depth) {
        stack.pop();
    }
    *unbraced = None;
}

pub(super) fn close_unbraced(
    stack: &[Frame],
    unbraced: &mut Option<Unbraced>,
    depth: usize,
    line: &str,
) {
    let Some(state) = unbraced.as_mut() else {
        return;
    };
    if opens_nested_header(line) {
        state.in_header = true;
        state.armed = false;
    }
    state.delimiters += paren_delta(line);
    let top_is_loop =
        matches!(stack.last(), Some(Frame::Loop { open, .. }) if *open == state.depth);
    let inside = depth > state.depth;
    if !top_is_loop {
        *unbraced = None;
        return;
    }
    if inside {
        return;
    }
    if state.in_header && state.delimiters <= 0 && statement_closed(line) {
        state.in_header = false;
    } else if !state.in_header && state.delimiters <= 0 && statement_closed(line) {
        state.armed = true;
    }
}

pub(super) fn with_loop_held(
    stack: &mut Vec<Frame>,
    hold: bool,
    body: impl FnOnce(&mut Vec<Frame>),
) {
    let held = if hold && matches!(stack.last(), Some(Frame::Loop { .. })) {
        stack.pop()
    } else {
        None
    };
    body(stack);
    if let Some(frame) = held {
        stack.push(frame);
    }
}

fn opens_nested_header(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.contains('{') || trimmed.ends_with(';') {
        return false;
    }
    matches!(
        first_word(trimmed),
        "if" | "else"
            | "elif"
            | "elsif"
            | "elseif"
            | "for"
            | "foreach"
            | "while"
            | "switch"
            | "catch"
            | "do"
    )
}

fn statement_closed(line: &str) -> bool {
    !line
        .trim_end()
        .ends_with(['(', ',', '.', '+', '-', '=', '?', ':', '{', '\\', '&', '|'])
}

fn paren_delta(line: &str) -> i32 {
    line.chars().fold(0, |delta, ch| match ch {
        '(' | '[' => delta + 1,
        ')' | ']' => delta - 1,
        _ => delta,
    })
}

pub(super) fn delimiter_delta(line: &str) -> i32 {
    line.chars().fold(0, |delta, ch| match ch {
        '(' | '[' | '{' => delta + 1,
        ')' | ']' | '}' => delta - 1,
        _ => delta,
    })
}

#[derive(Default)]
pub(super) struct IndentHold {
    delimiters: i32,
    for_header: bool,
    backslash: bool,
}

impl IndentHold {
    pub(super) fn continued(&self) -> bool {
        self.delimiters != 0 || self.backslash
    }

    pub(super) fn for_iterable(&self) -> bool {
        self.for_header
    }

    pub(super) fn advance(&mut self, line: &str, opened_for: bool) {
        let next = (self.delimiters + delimiter_delta(line)).max(0);
        if self.delimiters == 0 && opened_for && next > 0 {
            self.for_header = true;
        }
        if next == 0 {
            self.for_header = false;
        }
        self.delimiters = next;
        self.backslash = line.trim_end().ends_with('\\');
    }
}
