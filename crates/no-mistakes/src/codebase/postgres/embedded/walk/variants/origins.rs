use crate::codebase::postgres::embedded::placeholders;

/// Rewrites generated markers while keeping authored SQL byte identities stable.
pub(super) fn rewrite(sql: &str, origins: &[u32], offset: u32, publish: bool) -> Vec<u32> {
    let mut out = Vec::new();
    let mut at = 0;
    let mut seen = 0;
    while let Some(relative) = sql[at..].find(placeholders::INTERNAL_PLACEHOLDER) {
        let marker = at + relative;
        out.extend_from_slice(&origins[at..marker]);
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
        out.extend(std::iter::repeat_n(origins[marker], width + count));
        at = after + digits;
    }
    out.extend_from_slice(&origins[at..]);
    out
}
