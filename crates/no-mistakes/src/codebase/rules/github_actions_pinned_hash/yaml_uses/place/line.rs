#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Place {
    Root,
    Jobs,
    Job,
    Steps,
    Step,
}

#[derive(Clone, Copy)]
pub(super) struct Level {
    pub(super) place: Place,
    pub(super) mapping: bool,
    pub(super) flow: bool,
    pub(super) indent: usize,
    pub(super) items_are_steps: bool,
    pub(super) root: bool,
}

impl Level {
    pub(super) fn root_map() -> Self {
        Self {
            place: Place::Root,
            mapping: true,
            flow: false,
            indent: 0,
            items_are_steps: false,
            root: true,
        }
    }

    pub(super) fn root_sequence(indent: usize, flow: bool) -> Self {
        Self {
            place: Place::Root,
            mapping: false,
            flow,
            indent,
            items_are_steps: true,
            root: true,
        }
    }
}

pub(super) fn child_place(place: Place, key: &str) -> Place {
    match key {
        "jobs" => Place::Jobs,
        "steps" => Place::Steps,
        _ if place == Place::Jobs => Place::Job,
        _ => Place::Root,
    }
}

pub(super) struct Split<'a> {
    pub dash_indent: usize,
    pub sequence: bool,
    pub key_indent: usize,
    pub body: &'a str,
}

pub(super) fn split_line(line: &str) -> Split<'_> {
    let mut count = 0usize;
    let mut indices = line.char_indices().peekable();
    while indices.peek().is_some_and(|(_, ch)| ch.is_whitespace()) {
        count += 1;
        indices.next();
    }
    let dash_indent = count;
    let mut sequence = false;
    if let Some((offset, '-')) = indices.peek().copied() {
        let rest = &line[offset + '-'.len_utf8()..];
        if rest.is_empty() || rest.starts_with(char::is_whitespace) {
            sequence = true;
            indices.next();
            count += 1;
            while indices.peek().is_some_and(|(_, ch)| ch.is_whitespace()) {
                count += 1;
                indices.next();
            }
        }
    }
    let body = match indices.peek() {
        Some((offset, _)) => &line[*offset..],
        None => "",
    };
    Split {
        dash_indent,
        sequence,
        key_indent: count,
        body,
    }
}

pub(super) fn is_block_key(text: &str) -> bool {
    let text = text.trim_start();
    if text.is_empty() || text.starts_with(['#', '{', '[', '|', '>', '*', '&']) {
        return false;
    }
    if let Some(quote) = text.chars().next().filter(|ch| *ch == '"' || *ch == '\'') {
        let bytes = text.as_bytes();
        let mut end = 1;
        while end < bytes.len() && bytes[end] != quote as u8 {
            end += 1;
        }
        if end >= bytes.len() {
            return false;
        }
        return text[end + 1..].trim_start().starts_with(':');
    }
    match text.find(':') {
        Some(index) => {
            let after = &text[index + 1..];
            after.is_empty() || after.starts_with([' ', '\t', '#'])
        }
        None => false,
    }
}
