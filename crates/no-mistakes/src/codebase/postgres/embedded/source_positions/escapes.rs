pub(super) fn continuation(raw: &str) -> Option<usize> {
    if raw.starts_with("\\\r\n") {
        Some(3)
    } else if raw.starts_with("\\\n") || raw.starts_with("\\\r") {
        Some(2)
    } else if raw.starts_with("\\\u{2028}") || raw.starts_with("\\\u{2029}") {
        Some(4)
    } else {
        None
    }
}

pub(super) fn width(raw: &str, raw_mode: bool) -> usize {
    if raw.starts_with("\r\n") {
        return 2;
    }
    if raw_mode || !raw.starts_with('\\') {
        return raw.chars().next().unwrap().len_utf8();
    }
    match raw.as_bytes()[1] {
        b'x' => 4,
        b'u' if raw.as_bytes().get(2) == Some(&b'{') => raw.find('}').unwrap() + 1,
        b'u' => {
            let high = u32::from_str_radix(&raw[2..6], 16).unwrap();
            if (0xD800..=0xDBFF).contains(&high)
                && raw.get(6..8) == Some("\\u")
                && raw
                    .get(8..12)
                    .and_then(|value| u32::from_str_radix(value, 16).ok())
                    .is_some_and(|value| (0xDC00..=0xDFFF).contains(&value))
            {
                12
            } else {
                6
            }
        }
        b'0'..=b'7' => {
            let maximum = if raw.as_bytes()[1] <= b'3' { 3 } else { 2 };
            1 + raw.as_bytes()[1..]
                .iter()
                .take(maximum)
                .take_while(|byte| matches!(byte, b'0'..=b'7'))
                .count()
        }
        _ => 1 + raw[1..].chars().next().unwrap().len_utf8(),
    }
}

pub(super) fn extra_characters(raw: &str, width: usize, raw_mode: bool) -> usize {
    // Oxc renders an unpaired UTF-16 surrogate, including `\u{D800}`, as
    // replacement plus four hex digits.
    if raw_mode || !raw.starts_with("\\u") {
        return 0;
    }
    let Some(unit) = unicode_escape_unit(raw, width) else {
        return 0;
    };
    usize::from((0xD800..=0xDFFF).contains(&unit)) * 4
}

fn unicode_escape_unit(raw: &str, width: usize) -> Option<u32> {
    if width == 6 {
        return Some(u32::from_str_radix(&raw[2..6], 16).unwrap());
    }
    if raw.as_bytes().get(2) != Some(&b'{') {
        return None;
    }
    Some(u32::from_str_radix(&raw[3..width - 1], 16).unwrap())
}
