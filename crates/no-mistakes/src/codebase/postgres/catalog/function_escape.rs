use std::str::Chars;

pub(super) fn unescape(inner: &str) -> String {
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\'' && chars.peek() == Some(&'\'') {
            chars.next();
            out.push('\'');
            continue;
        }
        if character != '\\' {
            out.push(character);
            continue;
        }
        out.push(decode(&mut chars));
    }
    out
}

fn decode(chars: &mut std::iter::Peekable<Chars<'_>>) -> char {
    let Some(next) = chars.next() else {
        return '\\';
    };
    match next {
        'n' => '\n',
        't' => '\t',
        'r' => '\r',
        'b' => '\u{0008}',
        'f' => '\u{000c}',
        '\\' => '\\',
        '\'' => '\'',
        'x' => hex_value(chars, 2).unwrap_or('x'),
        'u' => hex_exact(chars, 4).unwrap_or('u'),
        'U' => hex_exact(chars, 8).unwrap_or('U'),
        digit if ('0'..='7').contains(&digit) => octal(chars, digit),
        other => other,
    }
}

fn hex_value(chars: &mut std::iter::Peekable<Chars<'_>>, max_digits: usize) -> Option<char> {
    let mut value = 0u32;
    let mut count = 0;
    while count < max_digits {
        let Some(digit) = chars.peek().and_then(|character| character.to_digit(16)) else {
            break;
        };
        chars.next();
        value = value * 16 + digit;
        count += 1;
    }
    (count > 0).then(|| char::from_u32(value)).flatten()
}

fn hex_exact(chars: &mut std::iter::Peekable<Chars<'_>>, digits: usize) -> Option<char> {
    let peeked = chars.clone().take(digits).collect::<String>();
    if peeked.len() != digits
        || !peeked
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return None;
    }
    for _ in 0..digits {
        chars.next();
    }
    char::from_u32(u32::from_str_radix(&peeked, 16).ok()?)
}

pub(super) fn unescape_unicode(inner: &str) -> String {
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        if chars.peek() == Some(&'\\') {
            chars.next();
            out.push('\\');
            continue;
        }
        if chars.peek() == Some(&'+') {
            chars.next();
            out.push(hex_exact(&mut chars, 6).unwrap_or('+'));
            continue;
        }
        out.push(hex_exact(&mut chars, 4).unwrap_or('\\'));
    }
    out
}

fn octal(chars: &mut std::iter::Peekable<Chars<'_>>, first: char) -> char {
    let mut value = first.to_digit(8).unwrap_or(0);
    for _ in 0..2 {
        let Some(digit) = chars.peek().and_then(|character| character.to_digit(8)) else {
            break;
        };
        chars.next();
        value = value * 8 + digit;
    }
    char::from_u32(value).unwrap_or(first)
}
