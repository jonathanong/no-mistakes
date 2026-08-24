pub(super) fn decode(value: &str, escape: char) -> Option<String> {
    let mut output = String::new();
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\'' && chars.next_if_eq(&'\'').is_some() {
            output.push('\'');
            continue;
        }
        if character != escape {
            output.push(character);
            continue;
        }
        if chars.next_if_eq(&escape).is_some() {
            output.push(escape);
            continue;
        }
        let long = chars.next_if_eq(&'+').is_some();
        let scalar = codepoint(&mut chars, if long { 6 } else { 4 })?;
        if (0xD800..=0xDBFF).contains(&scalar) && !long {
            if chars.next()? != escape {
                return None;
            }
            let low = codepoint(&mut chars, 4)?;
            if !(0xDC00..=0xDFFF).contains(&low) {
                return None;
            }
            let combined = 0x10000 + ((scalar - 0xD800) << 10) + (low - 0xDC00);
            output.push(char::from_u32(combined)?);
        } else {
            output.push(char::from_u32(scalar)?);
        }
    }
    Some(output)
}

fn codepoint(chars: &mut impl Iterator<Item = char>, digits: usize) -> Option<u32> {
    let codepoint = chars.take(digits).collect::<String>();
    (codepoint.len() == digits && codepoint.chars().all(|digit| digit.is_ascii_hexdigit()))
        .then(|| u32::from_str_radix(&codepoint, 16).ok())
        .flatten()
}

#[cfg(test)]
mod tests;
