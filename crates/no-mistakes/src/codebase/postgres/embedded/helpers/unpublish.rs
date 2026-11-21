use crate::codebase::postgres::embedded::placeholders::{
    INTERNAL_BIND, INTERNAL_PLACEHOLDER, PLACEHOLDER_MARKER,
};
use crate::codebase::postgres::embedded::SqlByteOrigins;

/// A published marker found at a recorded position: where it starts and how
/// many digits follow `sql_placeholder_`.
struct Marker {
    start: usize,
    digits: usize,
    public: &'static str,
    internal: &'static str,
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
        let digits_start = marker.start + marker.public.len();
        out.push_str(marker.internal);
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
    origins: &SqlByteOrigins,
) -> Option<Vec<u32>> {
    if origins.len() != text.len() {
        return None;
    }
    let mut out = Vec::with_capacity(origins.len());
    let mut at = 0;
    for marker in locate(text, placeholders)? {
        push_range(&mut out, origins, at, marker.start);
        let width = marker.internal.len() + marker.digits;
        out.extend(std::iter::repeat_n(origins.at(marker.start), width));
        at = marker.start + marker.public.len() + marker.digits;
    }
    push_range(&mut out, origins, at, origins.len());
    Some(out)
}

fn push_range(out: &mut Vec<u32>, origins: &SqlByteOrigins, from: usize, to: usize) {
    out.extend((from..to).map(|index| origins.at(index)));
}

pub(crate) fn unpublish_invocations(
    text: &str,
    placeholders: &[(u32, u32)],
    invocations: super::super::site_identity::Invocations,
) -> Option<super::super::site_identity::Invocations> {
    if invocations.is_empty() {
        return Some(invocations);
    }
    let markers = locate(text, placeholders)?;
    Some(invocations.remap(|at| {
        let count = markers.partition_point(|marker| marker.start < at);
        let delta: isize = markers[..count]
            .iter()
            .map(|marker| marker.internal.len() as isize - marker.public.len() as isize)
            .sum();
        at.saturating_add_signed(delta)
    }))
}

fn locate(text: &str, placeholders: &[(u32, u32)]) -> Option<Vec<Marker>> {
    let mut found = Vec::with_capacity(placeholders.len());
    let (mut line, mut column) = (1u32, 1u32);
    let mut offset = 0;
    for &target in placeholders {
        offset += advance_to(&text[offset..], &mut line, &mut column, target)?;
        let (public, internal) = if text[offset..].starts_with(PLACEHOLDER_MARKER) {
            (PLACEHOLDER_MARKER, INTERNAL_PLACEHOLDER)
        } else {
            ("$", INTERNAL_BIND)
        };
        let after = text[offset..].strip_prefix(public)?;
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            return None;
        }
        found.push(Marker {
            start: offset,
            digits,
            public,
            internal,
        });
        // The published width keeps the cursor aligned with recorded coordinates.
        column += (public.len() + digits) as u32;
        offset += public.len() + digits;
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

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "unpublish_tests.rs"]
mod tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "unpublish_bind_tests.rs"]
mod bind_tests;
