/// Generated interpolation markers must be distinguishable from user-authored
/// SQL that happens to contain the public `sql_placeholder_` substring.
pub(super) const PLACEHOLDER_MARKER: &str = "sql_placeholder_";
pub(super) const INTERNAL_PLACEHOLDER: &str = "\u{10FFFF}sqlph_";

pub(super) fn internal_placeholder(index: usize) -> String {
    format!("{INTERNAL_PLACEHOLDER}{index}")
}

pub(super) fn publish_placeholders(text: String) -> String {
    text.replace(INTERNAL_PLACEHOLDER, PLACEHOLDER_MARKER)
}

/// Replace internal markers while retaining the exact SQL positions of generated values.
pub(super) fn publish_placeholders_with_positions(text: String) -> (String, Vec<(u32, u32)>) {
    let mut output = String::with_capacity(text.len());
    let mut positions = Vec::new();
    let mut line = 1;
    let mut column = 1;
    let mut rest = text.as_str();
    while let Some(at) = rest.find(INTERNAL_PLACEHOLDER) {
        let prefix = &rest[..at];
        output.push_str(prefix);
        advance_position(prefix, &mut line, &mut column);
        let after = &rest[at + INTERNAL_PLACEHOLDER.len()..];
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        let marker = format!("{PLACEHOLDER_MARKER}{}", &after[..digits]);
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

pub(super) fn count_placeholders(text: &str) -> u32 {
    text.matches(INTERNAL_PLACEHOLDER).count() as u32
}

/// Shift internally generated placeholders in `text` by `offset`. User text
/// that contains the public `sql_placeholder_` spelling is left unchanged.
pub(super) fn renumber_placeholders(text: &str, offset: u32) -> String {
    if offset == 0 {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut seen = 0u32;
    while let Some(position) = rest.find(INTERNAL_PLACEHOLDER) {
        out.push_str(&rest[..position]);
        let after_marker = &rest[position + INTERNAL_PLACEHOLDER.len()..];
        let digits = after_marker.bytes().take_while(u8::is_ascii_digit).count();
        seen += 1;
        out.push_str(INTERNAL_PLACEHOLDER);
        out.push_str(&(offset + seen).to_string());
        rest = &after_marker[digits..];
    }
    out.push_str(rest);
    out
}
