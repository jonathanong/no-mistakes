mod delimiter;
mod unquote;

pub(crate) use delimiter::{
    ascii_document, char_delimiter, char_delimiter_quoted, line_opener, DelimiterStyle,
};
pub(crate) use unquote::unquote;

pub(crate) struct Document {
    pub delimiter: String,
    pub strip_tabs: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operator {
    HereString,
    Document { strip_tabs: bool },
}

impl Operator {
    pub(crate) fn width(self) -> usize {
        match self {
            Self::HereString => 3,
            Self::Document { strip_tabs } => 2 + usize::from(strip_tabs),
        }
    }
}

pub(crate) fn operator<T: Copy + Eq + From<u8>>(input: &[T], arithmetic: bool) -> Option<Operator> {
    let less = T::from(b'<');
    if !input.starts_with(&[less, less]) {
        return None;
    }
    if input.get(2) == Some(&less) {
        return Some(Operator::HereString);
    }
    (!arithmetic).then(|| Operator::Document {
        strip_tabs: input.get(2) == Some(&T::from(b'-')),
    })
}

fn first_document(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut arithmetic = 0;
    let mut index = 0;
    while index < bytes.len() {
        let input = &bytes[index..];
        if input.starts_with(b"$((") {
            arithmetic += 1;
            index += 3;
        } else if arithmetic > 0 && input.starts_with(b"))") {
            arithmetic -= 1;
            index += 2;
        } else {
            match operator(input, arithmetic > 0) {
                Some(Operator::Document { .. }) => return Some(index),
                Some(Operator::HereString) => index += 3,
                None => index += 1,
            }
        }
    }
    None
}

#[derive(Clone, Copy)]
pub(crate) enum CarriageReturn {
    Keep,
    BeforeNewline,
    StripTrailing,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Body {
    pub end: usize,
    pub newlines: usize,
    pub terminated: bool,
}

pub(crate) fn skip_body<T: Copy + Eq + From<u8>>(
    input: &[T],
    mut index: usize,
    delimiter: impl Iterator<Item = T> + Clone,
    strip_tabs: bool,
    carriage_return: CarriageReturn,
) -> Body {
    let mut newlines = 0;
    while index < input.len() {
        let rest = &input[index..];
        let width = rest
            .iter()
            .position(|ch| *ch == T::from(b'\n'))
            .unwrap_or(rest.len());
        let has_newline = width < rest.len();
        index += width + usize::from(has_newline);
        newlines += usize::from(has_newline);
        let mut line = &rest[..width];
        if matches!(carriage_return, CarriageReturn::StripTrailing)
            || (has_newline && matches!(carriage_return, CarriageReturn::BeforeNewline))
        {
            line = line.strip_suffix(&[T::from(b'\r')]).unwrap_or(line);
        }
        if line
            .iter()
            .copied()
            .skip_while(|ch| strip_tabs && *ch == T::from(b'\t'))
            .eq(delimiter.clone())
        {
            return Body {
                end: index,
                newlines,
                terminated: true,
            };
        }
    }
    Body {
        end: index,
        newlines,
        terminated: false,
    }
}

pub(crate) fn trimmed_body_lines<'a>(
    lines: &'a [&'a str],
    document: &Document,
) -> (&'a [&'a str], usize) {
    let end = lines
        .iter()
        .position(|line| line.trim() == document.delimiter)
        .unwrap_or(lines.len());
    (&lines[..end], (end + 1).min(lines.len()))
}

#[cfg(test)]
mod tests;
