use std::iter::Peekable;
use std::str::Bytes;

const BYTE_ESCAPES: &[(u8, u8)] = &[
    (b'a', 7),
    (b'b', 8),
    (b'e', 27),
    (b'E', 27),
    (b'f', 12),
    (b'n', b'\n'),
    (b'r', b'\r'),
    (b't', b'\t'),
    (b'v', 11),
    (b'\\', b'\\'),
    (b'\'', b'\''),
    (b'"', b'"'),
    (b'?', b'?'),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Quote {
    Unquoted,
    Single,
    Double,
    Ansi,
}

pub(super) fn unquote(raw: &str) -> Option<String> {
    let mut bytes = raw.bytes().peekable();
    let mut delimiter = Vec::new();
    let mut quote = Quote::Unquoted;
    while let Some(byte) = bytes.next() {
        match (quote, byte) {
            (Quote::Unquoted, b'$') if bytes.next_if_eq(&b'\'').is_some() => quote = Quote::Ansi,
            (Quote::Unquoted, b'\'') => quote = Quote::Single,
            (Quote::Unquoted, b'"') => quote = Quote::Double,
            (Quote::Single | Quote::Ansi, b'\'') | (Quote::Double, b'"') => quote = Quote::Unquoted,
            (Quote::Ansi, b'\\') => append_ansi_escape(&mut bytes, &mut delimiter)?,
            (Quote::Unquoted, b'\\') => append_escaped_byte(bytes.next()?, &mut delimiter),
            (Quote::Double, b'\\') => {
                match bytes.next_if(|byte| matches!(byte, b'$' | b'`' | b'"' | b'\\' | b'\n')) {
                    Some(escaped) => append_escaped_byte(escaped, &mut delimiter),
                    None => delimiter.push(byte),
                }
            }
            _ => delimiter.push(byte),
        }
    }
    (quote == Quote::Unquoted).then(|| String::from_utf8_lossy(&delimiter).into_owned())
}

fn append_escaped_byte(byte: u8, delimiter: &mut Vec<u8>) {
    if byte != b'\n' {
        delimiter.push(byte);
    }
}

fn append_ansi_escape(bytes: &mut Peekable<Bytes<'_>>, delimiter: &mut Vec<u8>) -> Option<()> {
    let escaped = bytes.next()?;
    let byte = match escaped {
        b'0'..=b'7' => {
            let first = u32::from(escaped - b'0');
            Some(digits(bytes, 8, 2, Some(first)).unwrap_or(first) as u8)
        }
        b'x' => digits(bytes, 16, 2, None).map(|value| value as u8),
        b'u' | b'U' => {
            let width = if escaped == b'u' { 4 } else { 8 };
            if let Some(character) = digits(bytes, 16, width, None).and_then(char::from_u32) {
                delimiter.extend_from_slice(character.encode_utf8(&mut [0; 4]).as_bytes());
                return Some(());
            }
            None
        }
        b'c' => bytes
            .next_if(u8::is_ascii)
            .map(|byte| if byte == b'?' { 127 } else { byte & 31 }),
        _ => BYTE_ESCAPES
            .iter()
            .find_map(|&(escape, value)| (escape == escaped).then_some(value)),
    };
    if let Some(byte) = byte {
        delimiter.push(byte);
    } else {
        delimiter.extend([b'\\', escaped]);
    }
    Some(())
}

fn digits(
    bytes: &mut Peekable<Bytes<'_>>,
    radix: u32,
    limit: usize,
    mut value: Option<u32>,
) -> Option<u32> {
    for _ in 0..limit {
        let Some(digit) = bytes
            .peek()
            .and_then(|byte| char::from(*byte).to_digit(radix))
        else {
            break;
        };
        bytes.next();
        value = Some(value.unwrap_or_default() * radix + digit);
    }
    value
}
