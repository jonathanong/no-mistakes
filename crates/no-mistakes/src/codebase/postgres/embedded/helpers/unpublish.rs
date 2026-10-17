use crate::codebase::postgres::embedded::placeholders::{INTERNAL_PLACEHOLDER, PLACEHOLDER_MARKER};

/// Inverse of `publish_placeholders_with_positions`: restores the internal
/// marker spelling at the recorded SQL positions, so user-authored text that
/// merely resembles a published marker is left alone. `None` means a recorded
/// position does not hold a published marker, and the caller must fail closed.
pub(crate) fn unpublish(text: &str, placeholders: &[(u32, u32)]) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let (mut line, mut column) = (1u32, 1u32);
    let mut rest = text;
    for &target in placeholders {
        let (skipped, remainder) = advance_to(rest, &mut line, &mut column, target)?;
        out.push_str(skipped);
        let after = remainder.strip_prefix(PLACEHOLDER_MARKER)?;
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            return None;
        }
        out.push_str(INTERNAL_PLACEHOLDER);
        out.push_str(&after[..digits]);
        // The published width keeps the cursor aligned with recorded coordinates.
        column += (PLACEHOLDER_MARKER.len() + digits) as u32;
        rest = &after[digits..];
    }
    out.push_str(rest);
    Some(out)
}

/// Splits `rest` at the character whose position is `target`.
fn advance_to<'a>(
    rest: &'a str,
    line: &mut u32,
    column: &mut u32,
    target: (u32, u32),
) -> Option<(&'a str, &'a str)> {
    for (offset, character) in rest.char_indices() {
        if (*line, *column) == target {
            return Some((&rest[..offset], &rest[offset..]));
        }
        if character == '\n' {
            *line += 1;
            *column = 1;
        } else {
            *column += 1;
        }
    }
    ((*line, *column) == target).then_some((rest, ""))
}
