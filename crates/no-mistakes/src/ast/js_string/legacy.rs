//! Oxc 0.153 paired only adjacent fixed-width Unicode escapes. The 0.155
//! builder also pairs braced escapes and escapes separated by continuations.
//! Preserve the earlier spelling-sensitive identity without mutating the AST.
//! Escape grammar follows Oxc's MIT-licensed lexer/unicode.rs and string.rs.

use std::borrow::Cow;

pub(super) fn needs_raw_identity(raw: &str) -> bool {
    if raw.contains("\\\n")
        || raw.contains("\\\r")
        || raw.contains("\\\u{2028}")
        || raw.contains("\\\u{2029}")
    {
        return true;
    }
    // Braced scalar escapes are common enough to avoid decoding them again.
    raw.match_indices("\\u{").any(|(start, _)| {
        let digits = &raw[start + 3..];
        let Some(end) = digits.find('}') else {
            return false;
        };
        let digits = digits[..end].trim_start_matches('0');
        u32::from_str_radix(digits, 16).is_ok_and(|point| (0xd800..=0xdfff).contains(&point))
    })
}

pub(super) fn from_raw<'a>(raw: &str, fallback: Cow<'a, str>, template: bool) -> Cow<'a, str> {
    // A newly paired surrogate is necessarily supplementary. Without one,
    // the ordinary JSStr bridge already reproduces every legacy code point.
    if !fallback.chars().any(|point| u32::from(point) > 0xffff) {
        return fallback;
    }
    let Some(points) = decode(raw, template) else {
        // Recovered invalid escapes still retain the parser's complete value.
        return fallback;
    };
    let lone = points.iter().any(|point| (0xd800..=0xdfff).contains(point));
    let mut text = String::with_capacity(raw.len());
    for point in points {
        match char::from_u32(point) {
            Some('\u{fffd}') if lone => text.push_str("\u{fffd}fffd"),
            Some(character) => text.push(character),
            None => super::push_surrogate(&mut text, point),
        }
    }
    if text == fallback {
        fallback
    } else {
        Cow::Owned(text)
    }
}

pub(crate) fn raw_has_lone_surrogate(raw: &str) -> Option<bool> {
    decode(raw, false).map(|points| {
        points
            .into_iter()
            .any(|point| (0xd800..=0xdfff).contains(&point))
    })
}

fn decode(mut raw: &str, template: bool) -> Option<Vec<u32>> {
    let mut points = Vec::new();
    while !raw.is_empty() {
        let character = raw.chars().next()?;
        raw = &raw[character.len_utf8()..];
        if character != '\\' {
            if character == '\r' && template {
                raw = raw.strip_prefix('\n').unwrap_or(raw);
                points.push(u32::from('\n'));
            } else {
                points.push(u32::from(character));
            }
            continue;
        }
        let escaped = raw.chars().next()?;
        raw = &raw[escaped.len_utf8()..];
        let point = match escaped {
            '\n' | '\u{2028}' | '\u{2029}' => continue,
            '\r' => {
                raw = raw.strip_prefix('\n').unwrap_or(raw);
                continue;
            }
            'n' => u32::from('\n'),
            'r' => u32::from('\r'),
            't' => u32::from('\t'),
            'b' => 8,
            'f' => 12,
            'v' => 11,
            'x' => take_hex(&mut raw, 2)?,
            'u' => unicode(&mut raw)?,
            '0'..='7' => {
                let mut value = u32::from(escaped) - u32::from('0');
                let limit = if escaped <= '3' { 2 } else { 1 };
                for _ in 0..limit {
                    let Some(byte @ b'0'..=b'7') = raw.as_bytes().first().copied() else {
                        break;
                    };
                    value = value * 8 + u32::from(byte - b'0');
                    raw = &raw[1..];
                }
                value
            }
            other => u32::from(other),
        };
        points.push(point);
    }
    Some(points)
}

fn unicode(raw: &mut &str) -> Option<u32> {
    if let Some(braced) = raw.strip_prefix('{') {
        let end = braced.find('}')?;
        let digits = &braced[..end];
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        let value = u32::from_str_radix(
            {
                let digits = digits.trim_start_matches('0');
                if digits.is_empty() {
                    "0"
                } else {
                    digits
                }
            },
            16,
        )
        .ok()?;
        if value > 0x10ffff {
            return None;
        }
        *raw = &braced[end + 1..];
        return Some(value);
    }
    let high = take_hex(raw, 4)?;
    if (0xd800..=0xdbff).contains(&high) {
        if let Some(mut next) = raw.strip_prefix("\\u") {
            if let Some(low) = take_hex(&mut next, 4) {
                if (0xdc00..=0xdfff).contains(&low) {
                    *raw = next;
                    return Some(0x10000 + (high - 0xd800) * 0x400 + low - 0xdc00);
                }
            }
        }
    }
    Some(high)
}

fn take_hex(raw: &mut &str, count: usize) -> Option<u32> {
    let digits = raw.get(..count)?;
    if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let value = u32::from_str_radix(digits, 16).ok()?;
    *raw = &raw[count..];
    Some(value)
}

#[cfg(test)]
mod tests;
