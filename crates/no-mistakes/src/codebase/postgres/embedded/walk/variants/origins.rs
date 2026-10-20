use crate::codebase::postgres::embedded::placeholders;

/// Rewrites generated markers while keeping authored SQL byte identities stable.
///
/// Callers pass one origin per SQL byte. Debug builds assert that length so a
/// short list cannot hide in release output. Release builds still index with
/// `get` and drop the missing span.
pub(crate) fn rewrite(sql: &str, origins: &[u32], offset: u32, publish: bool) -> Vec<u32> {
    debug_assert_eq!(origins.len(), sql.len());
    rewrite_impl(sql, origins, offset, publish)
}

fn rewrite_impl(sql: &str, origins: &[u32], offset: u32, publish: bool) -> Vec<u32> {
    let mut out = Vec::new();
    let mut at = 0;
    let mut seen = 0;
    while let Some(relative) = sql[at..].find(placeholders::INTERNAL_PLACEHOLDER) {
        let marker = at + relative;
        if let Some(chunk) = origins.get(at..marker) {
            out.extend_from_slice(chunk);
        }
        let after = marker + placeholders::INTERNAL_PLACEHOLDER.len();
        let digits = sql[after..].bytes().take_while(u8::is_ascii_digit).count();
        seen += 1;
        let width = if publish {
            placeholders::PLACEHOLDER_MARKER.len()
        } else {
            placeholders::INTERNAL_PLACEHOLDER.len()
        };
        let count = if offset == 0 {
            digits
        } else {
            (offset + seen).to_string().len()
        };
        if let Some(origin) = origins.get(marker) {
            out.extend(std::iter::repeat_n(*origin, width + count));
        }
        at = after + digits;
    }
    if let Some(tail) = origins.get(at..) {
        out.extend_from_slice(tail);
    }
    out
}

#[cfg(test)]
#[path = "origins_tests.rs"]
mod origins_tests;
