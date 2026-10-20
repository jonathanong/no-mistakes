//! End-of-walk step: no hole marker may reach a public fact.
//!
//! A call that still holds markers is reduced to exactly what the walk
//! produced before imported helpers were modeled: `Dynamic`, keeping only a
//! verified leading statement. When the whole call could be complete, the
//! marker-bearing form is kept aside for the projection step.

use super::sentinel;
use super::split::coordinate;
use super::summary::{ImportedSite, PendingImportedSql};
use crate::codebase::postgres::embedded::relative::PendingRelativeCall;
use crate::codebase::postgres::embedded::walk::resolve::has_known_leading_statement;
use crate::codebase::postgres::embedded::EmbeddedSqlVariant;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlFragment, EmbeddedSqlKind};
use std::collections::BTreeMap;

pub(crate) struct Walked<'a> {
    pub(crate) calls: &'a mut [EmbeddedSqlCall],
    pub(crate) call_spans: &'a [(u32, u32)],
    pub(crate) pending_calls: &'a mut [PendingRelativeCall],
    pub(crate) pending_spans: &'a BTreeMap<u32, (u32, u32)>,
    pub(crate) fragments: &'a mut [EmbeddedSqlFragment],
    pub(crate) fragment_variants: &'a mut [Vec<EmbeddedSqlVariant>],
}

pub(crate) fn finalize(walked: Walked<'_>, sites: &[ImportedSite]) -> Vec<PendingImportedSql> {
    let mut pending = Vec::new();
    for (call, span) in walked.calls.iter_mut().zip(walked.call_spans) {
        pending.extend(take_pending(call, *span, sites));
    }
    for relative in walked.pending_calls.iter_mut() {
        let span = walked.pending_spans.get(&relative.seq).copied();
        pending.extend(span.and_then(|span| take_pending(&mut relative.call, span, sites)));
    }
    for fragment in walked.fragments.iter_mut() {
        if !fragment.sql_text.as_deref().is_some_and(sentinel::contains) {
            continue;
        }
        let text = fragment.sql_text.take().unwrap_or_default();
        // Defensive invariant: positions are normally empty at this point.
        let Some((kept, cut)) = trimmed_prefix(&text) else {
            fragment.recovered_placeholder_positions.clear();
            continue;
        };
        fragment
            .recovered_placeholder_positions
            .retain(|position| *position < cut);
        fragment.sql_text = Some(kept);
    }
    for variants in walked.fragment_variants.iter_mut() {
        if variants
            .iter()
            .any(|variant| sentinel::contains(&variant.sql_text))
        {
            variants.clear();
        }
    }
    pending
}

fn take_pending(
    call: &mut EmbeddedSqlCall,
    span: (u32, u32),
    sites: &[ImportedSite],
) -> Option<PendingImportedSql> {
    let in_text = call.sql_text.as_deref().is_some_and(sentinel::contains);
    let in_variants = call
        .variants
        .iter()
        .any(|variant| sentinel::contains(&variant.sql_text));
    if !in_text && !in_variants {
        return None;
    }
    // Either the call's own text is a complete statement, or each of its
    // complete alternatives is and the legacy text holds no hole.
    let complete = if call.variants.is_empty() {
        in_text && call.kind != EmbeddedSqlKind::Dynamic
    } else {
        !in_text
    };
    let template = call.clone();
    scrub(call);
    if !complete {
        return None;
    }
    let texts = template
        .sql_text
        .iter()
        .chain(template.variants.iter().map(|variant| &variant.sql_text));
    let sites = used_sites(texts, sites)?;
    Some(PendingImportedSql {
        span,
        template,
        sites,
    })
}

fn used_sites<'a>(
    texts: impl Iterator<Item = &'a String>,
    sites: &[ImportedSite],
) -> Option<Vec<(usize, ImportedSite)>> {
    let mut used = Vec::new();
    for text in texts {
        for (_, index) in sentinel::find_all(text) {
            if used.iter().all(|(seen, _)| *seen != index) {
                used.push((index, sites.get(index)?.clone()));
            }
        }
    }
    Some(used)
}

/// What a call with an unrecoverable helper holds: only the verified leading
/// statement that precedes the first hole.
fn scrub(call: &mut EmbeddedSqlCall) {
    call.variants.clear();
    call.kind = EmbeddedSqlKind::Dynamic;
    if !call.sql_text.as_deref().is_some_and(sentinel::contains) {
        return;
    }
    let text = call.sql_text.take().unwrap_or_default();
    let Some((kept, cut)) = trimmed_prefix(&text) else {
        call.sql_source_positions.clear();
        call.recovered_placeholder_positions.clear();
        return;
    };
    call.sql_source_positions
        .retain(|position| (position.sql_line, position.sql_column) < cut);
    call.recovered_placeholder_positions
        .retain(|position| *position < cut);
    call.sql_text = Some(kept);
}

/// Prefix before the first hole, plus the SQL coordinate where that prefix ends.
fn trimmed_prefix(text: &str) -> Option<(String, (u32, u32))> {
    let prefix = sentinel::prefix(text);
    let kept = keep_prefix(text)?;
    Some((kept, coordinate(text, prefix.len())))
}

/// The prefix before the first hole, when it is a known statement.
fn keep_prefix(text: &str) -> Option<String> {
    let prefix = sentinel::prefix(text);
    has_known_leading_statement(prefix).then(|| prefix.to_string())
}
