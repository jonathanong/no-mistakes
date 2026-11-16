use crate::codebase::postgres::embedded::placeholders;

/// Rewrites generated markers while keeping authored SQL byte identities stable.
///
/// Callers pass one origin per SQL byte. Debug builds assert that length so a
/// short list cannot hide in release output. Release builds still index with
/// `get` and drop the missing span.
pub(crate) fn rewrite(
    sql: &str,
    origins: &[u32],
    offset: impl Into<placeholders::Counts>,
    publish: bool,
) -> Vec<u32> {
    debug_assert_eq!(origins.len(), sql.len());
    rewrite_impl(sql, origins, offset.into(), publish)
}

fn rewrite_impl(
    sql: &str,
    origins: &[u32],
    offset: placeholders::Counts,
    publish: bool,
) -> Vec<u32> {
    let mut out = Vec::new();
    let mut at = 0;
    let mut seen = placeholders::Counts::default();
    while let Some((relative, internal, public)) = placeholders::markers(&sql[at..]).next() {
        let marker = at + relative;
        if let Some(chunk) = origins.get(at..marker) {
            out.extend_from_slice(chunk);
        }
        let after = marker + internal.len();
        let digits = sql[after..].bytes().take_while(u8::is_ascii_digit).count();
        let ordinal = seen.next(internal);
        let width = if publish {
            public.len()
        } else {
            internal.len()
        };
        let count = if offset.shift(internal) == 0 {
            digits
        } else {
            (offset.shift(internal) + ordinal).to_string().len()
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
