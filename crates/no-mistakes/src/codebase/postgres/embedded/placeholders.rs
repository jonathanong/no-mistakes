/// Generated interpolation markers must be distinguishable from user-authored
/// SQL that happens to contain the public `sql_placeholder_` substring.
pub(super) const PLACEHOLDER_MARKER: &str = "sql_placeholder_";
const INTERNAL_PLACEHOLDER: &str = "\u{10FFFF}sqlph_";

pub(super) fn internal_placeholder(index: usize) -> String {
    format!("{INTERNAL_PLACEHOLDER}{index}")
}

pub(super) fn publish_placeholders(text: String) -> String {
    text.replace(INTERNAL_PLACEHOLDER, PLACEHOLDER_MARKER)
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
