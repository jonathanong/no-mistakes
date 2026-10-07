//! Continuation segments inherit the first E literal's escape state.
pub(super) fn decode(value: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut chars = value.chars().peekable();
    while let Some(mut character) = chars.next() {
        if character == '\\' {
            character = chars.next()?;
            character = match character {
                'b' => '\u{0008}',
                'f' => '\u{000c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'x' => {
                    let digits = take_digits(&mut chars, 16, 2);
                    if !digits.is_empty() {
                        bytes.push(u8::from_str_radix(&digits, 16).unwrap());
                        continue;
                    }
                    'x'
                }
                'u' | 'U' => {
                    let count = if character == 'u' { 4 } else { 8 };
                    let digits = take_digits(&mut chars, 16, count);
                    if digits.len() != count {
                        return None;
                    }
                    char::from_u32(u32::from_str_radix(&digits, 16).unwrap())?
                }
                digit @ '0'..='7' => {
                    let digits = format!("{digit}{}", take_digits(&mut chars, 8, 2));
                    bytes.push((u16::from_str_radix(&digits, 8).unwrap() & 255) as u8);
                    continue;
                }
                other => other,
            };
        }
        bytes.extend_from_slice(character.encode_utf8(&mut [0; 4]).as_bytes());
    }
    let value = String::from_utf8(bytes).ok()?;
    (!value.contains('\0')).then_some(value)
}

// Callers bound the digit count so numeric conversion cannot overflow.
fn take_digits(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    radix: u32,
    max: usize,
) -> String {
    let mut digits = String::new();
    while digits.len() < max && chars.peek().is_some_and(|value| value.is_digit(radix)) {
        digits.push(chars.next().unwrap());
    }
    digits
}

#[cfg(test)]
mod tests;
