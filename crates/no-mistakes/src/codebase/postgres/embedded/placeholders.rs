/// Generated interpolation markers must be distinguishable from user-authored
/// SQL that happens to contain the public `sql_placeholder_` substring.
pub(super) const PLACEHOLDER_MARKER: &str = "sql_placeholder_";
pub(crate) const INTERNAL_PLACEHOLDER: &str = "\u{10FFFF}sqlph_";

pub(crate) const INTERNAL_BIND: &str = "\u{10FFFF}sqlbv_";

pub(crate) fn internal_bind(index: usize) -> String {
    format!("{INTERNAL_BIND}{index}")
}

pub(crate) fn markers(
    text: &str,
) -> impl Iterator<Item = (usize, &'static str, &'static str)> + '_ {
    text.match_indices('\u{10FFFF}').filter_map(|(at, _)| {
        if text[at..].starts_with(INTERNAL_PLACEHOLDER) {
            Some((at, INTERNAL_PLACEHOLDER, PLACEHOLDER_MARKER))
        } else if text[at..].starts_with(INTERNAL_BIND) {
            Some((at, INTERNAL_BIND, "$"))
        } else {
            None
        }
    })
}

pub(super) fn internal_placeholder(index: usize) -> String {
    format!("{INTERNAL_PLACEHOLDER}{index}")
}

pub(super) fn publish_placeholders(text: String) -> String {
    text.replace(INTERNAL_PLACEHOLDER, PLACEHOLDER_MARKER)
        .replace(INTERNAL_BIND, "$")
}

/// Replace internal markers while retaining the exact SQL positions of generated values.
pub(crate) fn publish_placeholders_with_positions(text: String) -> (String, Vec<(u32, u32)>) {
    let mut output = String::with_capacity(text.len());
    let mut positions = Vec::new();
    let mut line = 1;
    let mut column = 1;
    let mut rest = text.as_str();
    while let Some((at, internal, public)) = markers(rest).next() {
        let prefix = &rest[..at];
        output.push_str(prefix);
        advance_position(prefix, &mut line, &mut column);
        let after = &rest[at + internal.len()..];
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        let marker = format!("{public}{}", &after[..digits]);
        positions.push((line, column));
        output.push_str(&marker);
        advance_position(&marker, &mut line, &mut column);
        rest = &after[digits..];
    }
    output.push_str(rest);
    (output, positions)
}

fn advance_position(text: &str, line: &mut u32, column: &mut u32) {
    for character in text.chars() {
        if character == '\n' {
            *line += 1;
            *column = 1;
        } else {
            *column += 1;
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Counts {
    pub(crate) all: u32,
    pub(crate) binds: u32,
}
impl Counts {
    pub(crate) fn next(&mut self, marker: &str) -> u32 {
        self.all += 1;
        if marker == INTERNAL_BIND {
            self.binds += 1;
            self.binds
        } else {
            self.all
        }
    }
    pub(crate) fn shift(self, marker: &str) -> u32 {
        if marker == INTERNAL_BIND {
            self.binds
        } else {
            self.all
        }
    }
}
impl From<u32> for Counts {
    fn from(value: u32) -> Self {
        Self {
            all: value,
            binds: value,
        }
    }
}
impl std::ops::AddAssign for Counts {
    fn add_assign(&mut self, other: Self) {
        self.all += other.all;
        self.binds += other.binds;
    }
}

pub(crate) fn count_placeholders(text: &str) -> Counts {
    let mut counts = Counts::default();
    for (_, internal, _) in markers(text) {
        counts.next(internal);
    }
    counts
}

/// Shift generated markers, counting bound values independently of identifier holes.
pub(crate) fn renumber_placeholders(text: &str, offset: impl Into<Counts>) -> String {
    let offset = offset.into();
    if offset == Counts::default() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut seen = Counts::default();
    while let Some((position, internal, _)) = markers(rest).next() {
        out.push_str(&rest[..position]);
        let after_marker = &rest[position + internal.len()..];
        let digits = after_marker.bytes().take_while(u8::is_ascii_digit).count();
        let ordinal = seen.next(internal);
        out.push_str(internal);
        out.push_str(&(offset.shift(internal) + ordinal).to_string());
        rest = &after_marker[digits..];
    }
    out.push_str(rest);
    out
}

pub(crate) fn parameterize_values(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let (mut cursor, mut binds) = (0, 0);
    for (at, internal, _) in markers(text) {
        let end = at
            + internal.len()
            + text[at + internal.len()..]
                .bytes()
                .take_while(u8::is_ascii_digit)
                .count();
        out.push_str(&text[cursor..at]);
        if super::helpers::sentinel::in_relation_slot(text, at) {
            out.push_str(&text[at..end]);
        } else {
            binds += 1;
            out.push_str(&internal_bind(binds));
        }
        cursor = end;
    }
    out.push_str(&text[cursor..]);
    out
}
