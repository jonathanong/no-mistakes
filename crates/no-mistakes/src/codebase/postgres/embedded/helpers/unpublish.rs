use crate::codebase::postgres::embedded::placeholders::{INTERNAL_PLACEHOLDER, PLACEHOLDER_MARKER};

/// A published marker found at a recorded position: where it starts and how
/// many digits follow `sql_placeholder_`.
struct Marker {
    start: usize,
    digits: usize,
}

/// Inverse of `publish_placeholders_with_positions`: restores the internal
/// marker spelling at the recorded SQL positions, so user-authored text that
/// merely resembles a published marker is left alone. `None` means a recorded
/// position does not hold a published marker, and the caller must fail closed.
pub(crate) fn unpublish(text: &str, placeholders: &[(u32, u32)]) -> Option<String> {
    let markers = locate(text, placeholders)?;
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    for marker in markers {
        out.push_str(&text[at..marker.start]);
        let digits_start = marker.start + PLACEHOLDER_MARKER.len();
        out.push_str(INTERNAL_PLACEHOLDER);
        out.push_str(&text[digits_start..digits_start + marker.digits]);
        at = digits_start + marker.digits;
    }
    out.push_str(&text[at..]);
    Some(out)
}

/// Per-byte physical origins for the text [`unpublish`] returns: a restored
/// marker keeps the origin of its first published byte, as `rewrite` does.
pub(crate) fn unpublish_origins(
    text: &str,
    placeholders: &[(u32, u32)],
    origins: &[u32],
) -> Option<Vec<u32>> {
    if origins.len() != text.len() {
        return None;
    }
    let mut out = Vec::with_capacity(origins.len());
    let mut at = 0;
    for marker in locate(text, placeholders)? {
        out.extend_from_slice(&origins[at..marker.start]);
        let width = INTERNAL_PLACEHOLDER.len() + marker.digits;
        out.extend(std::iter::repeat_n(origins[marker.start], width));
        at = marker.start + PLACEHOLDER_MARKER.len() + marker.digits;
    }
    out.extend_from_slice(&origins[at..]);
    Some(out)
}

fn locate(text: &str, placeholders: &[(u32, u32)]) -> Option<Vec<Marker>> {
    let mut found = Vec::with_capacity(placeholders.len());
    let (mut line, mut column) = (1u32, 1u32);
    let mut offset = 0;
    for &target in placeholders {
        offset += advance_to(&text[offset..], &mut line, &mut column, target)?;
        let after = text[offset..].strip_prefix(PLACEHOLDER_MARKER)?;
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            return None;
        }
        found.push(Marker {
            start: offset,
            digits,
        });
        // The published width keeps the cursor aligned with recorded coordinates.
        column += (PLACEHOLDER_MARKER.len() + digits) as u32;
        offset += PLACEHOLDER_MARKER.len() + digits;
    }
    Some(found)
}

/// Byte distance from the start of `rest` to the character at `target`.
fn advance_to(rest: &str, line: &mut u32, column: &mut u32, target: (u32, u32)) -> Option<usize> {
    for (offset, character) in rest.char_indices() {
        if (*line, *column) == target {
            return Some(offset);
        }
        if character == '\n' {
            *line += 1;
            *column = 1;
        } else {
            *column += 1;
        }
    }
    ((*line, *column) == target).then_some(rest.len())
}
