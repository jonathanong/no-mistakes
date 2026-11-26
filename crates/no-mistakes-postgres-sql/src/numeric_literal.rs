/// PostgreSQL permits separators between radix digits and once immediately after a radix prefix.
pub(super) fn integer(text: &str) -> Result<u64, std::num::ParseIntError> {
    let (digits, radix) = match text.get(..2).map(str::to_ascii_lowercase).as_deref() {
        Some("0x") => (&text[2..], 16),
        Some("0o") => (&text[2..], 8),
        Some("0b") => (&text[2..], 2),
        _ => (text, 10),
    };
    let digits = if radix != 10 {
        digits.strip_prefix('_').unwrap_or(digits)
    } else {
        digits
    };
    let chars: Vec<char> = digits.chars().collect();
    if chars.iter().enumerate().any(|(index, character)| {
        *character == '_'
            && (index == 0
                || index + 1 == chars.len()
                || !chars[index - 1].is_digit(radix)
                || !chars[index + 1].is_digit(radix))
    }) {
        return u64::from_str_radix("invalid", radix);
    }
    u64::from_str_radix(&digits.replace('_', ""), radix)
}
