//! End-of-walk step: no hole marker may reach a public fact.
//! Unfilled calls stay dynamic. Identifier holes keep `sql_placeholder_N` and
//! the rest of the statement. An unresolved fragment ends the verified prefix.

use super::sentinel;
use super::split::coordinate;
use super::summary::{ImportedSite, PendingImportedSql};
use crate::codebase::postgres::embedded::relative::PendingRelativeCall;
use crate::codebase::postgres::embedded::EmbeddedSqlVariant;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlFragment, EmbeddedSqlKind};
use std::collections::BTreeMap;

mod policy;
use policy::trimmed_prefix;

pub(crate) struct Walked<'a> {
    pub(crate) calls: &'a mut [EmbeddedSqlCall],
    pub(crate) call_spans: &'a [(u32, u32)],
    pub(crate) pending_calls: &'a mut [PendingRelativeCall],
    pub(crate) pending_spans: &'a BTreeMap<u32, (u32, u32)>,
    pub(crate) fragments: &'a mut [EmbeddedSqlFragment],
    pub(crate) fragment_variants: &'a mut [Vec<EmbeddedSqlVariant>],
}

pub(crate) fn finalize(
    walked: Walked<'_>,
    sites: &[ImportedSite],
    groups: &crate::fx::FxHashMap<usize, u32>,
) -> Vec<PendingImportedSql> {
    let mut pending = Vec::new();
    for (call, span) in walked.calls.iter_mut().zip(walked.call_spans) {
        pending.extend(take_pending(call, *span, sites, groups));
    }
    for relative in walked.pending_calls.iter_mut() {
        let span = walked.pending_spans.get(&relative.seq).copied();
        pending.extend(span.and_then(|span| take_pending(&mut relative.call, span, sites, groups)));
    }
    for fragment in walked.fragments.iter_mut() {
        if !fragment.sql_text.as_deref().is_some_and(sentinel::contains) {
            continue;
        }
        let text = fragment.sql_text.take().unwrap_or_default();
        // Defensive invariant (#2287): fragment placeholder positions are normally
        // empty here (assigned later in embedded.rs). Keep the trim so positions
        // never point past the cut; covered by
        // trimmed_fragment_drops_placeholder_positions_past_the_hole.
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
    groups: &crate::fx::FxHashMap<usize, u32>,
) -> Option<PendingImportedSql> {
    let in_text = call.sql_text.as_deref().is_some_and(sentinel::contains);
    let in_variants = call
        .variants
        .iter()
        .any(|variant| sentinel::contains(&variant.sql_text));
    if !in_text && !in_variants {
        return None;
    }
    // A deterministic singleton mirrors complete SQL in both representations.
    // Keep only the variant: splice fills variants and otherwise clones raw SQL.
    let singleton = in_text
        && call.kind != EmbeddedSqlKind::Dynamic
        && call.variants.len() == 1
        && !call.variants_truncated()
        && call.sql_text.as_deref() == Some(call.variants[0].sql_text.as_str());
    let complete = if call.variants.is_empty() {
        in_text && call.kind != EmbeddedSqlKind::Dynamic
    } else {
        !in_text || singleton
    };
    let mut template = call.clone();
    if singleton {
        template.sql_text = None;
        template.sql_source_positions.clear();
        template.recovered_placeholder_positions.clear();
    }
    publish_call(call, sites);
    if !complete {
        return None;
    }
    let texts = template
        .sql_text
        .iter()
        .chain(template.variants.iter().map(|variant| &variant.sql_text));
    let sites = used_sites(texts, sites)?;
    let correlation = groups
        .iter()
        .filter(|(index, _)| sites.iter().any(|(seen, _)| seen == *index))
        .map(|(index, group)| (*index, *group))
        .collect();
    Some(PendingImportedSql {
        span,
        template,
        sites,
        correlation,
    })
}

fn publish_call(call: &mut EmbeddedSqlCall, sites: &[ImportedSite]) {
    let source = call.sql_text.clone().or_else(|| {
        call.variants
            .first()
            .map(|variant| variant.sql_text.clone())
    });
    call.variants.clear();
    call.kind = EmbeddedSqlKind::Dynamic;
    let Some(text) = source else {
        call.sql_text = None;
        return;
    };
    if let Some((range, _)) = sentinel::find_all(&text).first() {
        let cut = coordinate(&text, range.start);
        call.sql_source_positions
            .retain(|position| (position.sql_line, position.sql_column) < cut);
        call.recovered_placeholder_positions
            .retain(|position| *position < cut);
    }
    let published = policy::publish(&text, sites);
    call.sql_text = Some(published).filter(|text| {
        let trimmed = text.trim();
        !trimmed.is_empty() && !placeholder_only(trimmed)
    });
}

/// A helper call with no surrounding statement is not public SQL.
fn placeholder_only(text: &str) -> bool {
    let mut rest = text;
    while let Some(stripped) = rest.strip_prefix("sql_placeholder_") {
        let digits = stripped.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            return false;
        }
        rest = stripped[digits..].trim_start();
        if rest.is_empty() {
            return true;
        }
    }
    false
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
