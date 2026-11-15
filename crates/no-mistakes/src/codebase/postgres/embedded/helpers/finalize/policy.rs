//! When a hole stays a public placeholder, and where a known prefix ends.
//!
//! A placeholder replaces an identifier hole (`LOCK TABLE`, `FROM`, `INTO`).
//! A whole unresolved fragment is not an identifier hole: restoring it glues
//! `sql_placeholder_N` onto the surrounding text and the statement fails to parse.

use super::super::sentinel;
use super::super::split::coordinate;
use crate::codebase::postgres::embedded::walk::resolve::has_known_leading_statement;

/// Prefix before the first hole, plus the SQL coordinate where that prefix ends.
pub(super) fn trimmed_prefix(text: &str) -> Option<(String, (u32, u32))> {
    let prefix = sentinel::prefix(text);
    let kept = keep_prefix(text)?;
    Some((kept, coordinate(text, prefix.len())))
}

/// A non-bare hole whose leading text is not a known statement, and whose hole
/// is an identifier slot. Other holes are scrubbed with the fragment.
pub(super) fn unfilled_statement(text: &str) -> bool {
    sentinel::contains(text)
        && !sentinel::is_bare(text)
        && keep_prefix(text).is_none()
        && sentinel::has_identifier_slot_hole(text)
}

/// The prefix before the first hole, when it is a known statement.
fn keep_prefix(text: &str) -> Option<String> {
    let prefix = sentinel::prefix(text);
    has_known_leading_statement(prefix).then(|| prefix.to_string())
}
