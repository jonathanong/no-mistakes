use super::params::parameter_names;
use super::{Frame, Header};

pub(super) struct Unbraced {
    pub(super) depth: usize,
    pub(super) delimiters: i32,
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

pub(super) fn close_unbraced(
    stack: &mut Vec<Frame>,
    unbraced: &mut Option<Unbraced>,
    depth: usize,
    line: &str,
) {
    let Some(state) = unbraced.as_mut() else {
        return;
    };
    state.delimiters += paren_delta(line);
    let top_is_loop =
        matches!(stack.last(), Some(Frame::Loop { open, .. }) if *open == state.depth);
    let inside = depth > state.depth;
    if !top_is_loop || (!inside && state.delimiters <= 0 && statement_closed(line)) {
        if top_is_loop {
            stack.pop();
        }
        *unbraced = None;
    }
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
