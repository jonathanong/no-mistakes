//! When a hole stays a public placeholder, and where a known prefix ends.
//!
//! An identifier or value hole stays `sql_placeholder_N`, and the text after
//! it stays too. A real fragment splice (an `.append` of an unresolved
//! fragment) ends the verified prefix until the fragment is resolved.

use super::super::sentinel;
use super::super::split::coordinate;
use super::super::summary::ImportedSite;
use crate::codebase::postgres::embedded::walk::{
    resolve::has_known_leading_statement, scope::has_diagnostic_leading_statement,
};
use std::ops::Range;

/// Replace identifier holes, stopping at an unresolved fragment splice.
pub(super) fn publish(text: &str, sites: &[ImportedSite]) -> String {
    let holes = sentinel::find_all(text);
    if holes.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    let mut index = 1usize;
    for (range, site) in &holes {
        out.push_str(&text[cursor..range.start]);
        if fragment_gap(text, range, sites.get(*site)) {
            return if has_diagnostic_leading_statement(&out) {
                out
            } else {
                String::new()
            };
        } else {
            out.push_str("sql_placeholder_");
            out.push_str(&index.to_string());
            index += 1;
        }
        cursor = range.end;
    }
    out.push_str(&text[cursor..]);
    out
}

fn fragment_gap(text: &str, range: &Range<usize>, site: Option<&ImportedSite>) -> bool {
    let Some(site) = site else {
        return false;
    };
    if sentinel::in_identifier_slot(text, range.start) || value_operand(text, range.start) {
        return false;
    }
    match site.constant {
        Some(true) => true,
        // An imported name is an identifier, even outside the keyword list.
        Some(false) => false,
        // A frozen-record field stays a placeholder. A helper call splices SQL.
        None => site.member.is_none(),
    }
}

fn value_operand(text: &str, hole_start: usize) -> bool {
    text[..hole_start].trim_end().ends_with('=')
}

/// Prefix before the first hole, plus the SQL coordinate where that prefix ends.
pub(super) fn trimmed_prefix(text: &str) -> Option<(String, (u32, u32))> {
    let prefix = sentinel::prefix(text);
    let kept = keep_prefix(text)?;
    Some((kept, coordinate(text, prefix.len())))
}

/// The prefix before the first hole, when it is a known statement.
fn keep_prefix(text: &str) -> Option<String> {
    let prefix = sentinel::prefix(text);
    has_known_leading_statement(prefix).then(|| prefix.to_string())
}
