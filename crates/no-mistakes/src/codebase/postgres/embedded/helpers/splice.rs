use super::sentinel;
use super::split::{compose, split};
use super::unpublish::{unpublish, unpublish_origins};
use crate::codebase::postgres::embedded::placeholders::publish_placeholders_with_positions;
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::postgres::embedded::walk::resolve::rewrite_origins;
use crate::codebase::postgres::embedded::{
    EmbeddedSqlCall, EmbeddedSqlKind, EmbeddedSqlVariant, MAX_EMBEDDED_SQL_VARIANTS,
};
use crate::fx::FxHashMap;

/// Every alternative text of each hole, by the site index the walk gave it.
type Holes = FxHashMap<usize, Vec<RecoveredSql>>;

/// Fills every hole of `template` and republishes it. A call whose own text
/// holds holes becomes a complete `Composed` call when every hole has one
/// text, and a call with finite alternatives otherwise; a call whose holes
/// sit in its complete alternatives keeps its kind and legacy text and gets
/// filled alternatives. Any gap, or more than
/// [`MAX_EMBEDDED_SQL_VARIANTS`] results, yields `None` so the scrubbed
/// `Dynamic` call stays in place.
pub(crate) fn splice(template: &EmbeddedSqlCall, holes: &Holes) -> Option<EmbeddedSqlCall> {
    if !template.variants.is_empty() {
        let mut variants = Vec::new();
        for variant in &template.variants {
            for filled in splice_variant(variant, holes)? {
                EmbeddedSqlVariant::push_unique(&mut variants, filled);
            }
        }
        return (variants.len() <= MAX_EMBEDDED_SQL_VARIANTS).then(|| EmbeddedSqlCall {
            variants,
            ..template.clone()
        });
    }
    let published = template.sql_text.as_deref()?;
    let first_line = match template.kind {
        EmbeddedSqlKind::Inline => template.line,
        _ => template.declaration_line.unwrap_or(template.line),
    };
    let sql = unpublish(published, &template.recovered_placeholder_positions)?;
    let caller = RecoveredSql {
        origins: vec![0; sql.len()],
        positions: template.sql_source_positions.clone(),
        line: first_line,
        sql,
    };
    let mut filled = fill_all(&caller, holes)?;
    if filled.len() > 1 {
        let mut variants = Vec::new();
        for joined in filled {
            EmbeddedSqlVariant::push_unique(&mut variants, publish_variant(joined, Vec::new()));
        }
        return Some(EmbeddedSqlCall {
            variants,
            sql_text: None,
            kind: EmbeddedSqlKind::Dynamic,
            declaration_line: None,
            sql_source_positions: Vec::new(),
            recovered_placeholder_positions: Vec::new(),
            ..template.clone()
        });
    }
    let joined = filled.pop()?;
    let (sql_text, recovered_placeholder_positions) =
        publish_placeholders_with_positions(joined.sql);
    Some(EmbeddedSqlCall {
        sql_text: Some(sql_text),
        kind: EmbeddedSqlKind::Composed,
        declaration_line: Some(joined.line),
        sql_source_positions: joined.positions,
        recovered_placeholder_positions,
        ..template.clone()
    })
}

fn splice_variant(variant: &EmbeddedSqlVariant, holes: &Holes) -> Option<Vec<EmbeddedSqlVariant>> {
    let positions = &variant.recovered_placeholder_positions;
    let caller = RecoveredSql {
        sql: unpublish(&variant.sql_text, positions)?,
        origins: unpublish_origins(&variant.sql_text, positions, &variant.sql_source_offsets)?,
        line: variant.line,
        positions: variant.sql_source_positions.clone(),
    };
    Some(
        fill_all(&caller, holes)?
            .into_iter()
            .map(|joined| publish_variant(joined, variant.append_sites.clone()))
            .collect(),
    )
}

fn publish_variant(joined: RecoveredSql, append_sites: Vec<u32>) -> EmbeddedSqlVariant {
    let sql_source_offsets = rewrite_origins(&joined.sql, &joined.origins, 0, true);
    let (sql_text, recovered_placeholder_positions) =
        publish_placeholders_with_positions(joined.sql);
    EmbeddedSqlVariant {
        sql_text,
        sql_source_offsets,
        line: joined.line,
        sql_source_positions: joined.positions,
        recovered_placeholder_positions,
        append_sites,
    }
}

/// `caller` with every hole replaced by each combination of its alternatives.
/// A hole that appears twice takes the same alternative both times.
fn fill_all(caller: &RecoveredSql, holes: &Holes) -> Option<Vec<RecoveredSql>> {
    let markers = sentinel::find_all(&caller.sql);
    let mut sites: Vec<usize> = markers.iter().map(|(_, site)| *site).collect();
    sites.sort_unstable();
    sites.dedup();
    let mut combos: Vec<Vec<&RecoveredSql>> = vec![Vec::new()];
    for site in &sites {
        let alternatives = holes.get(site).filter(|texts| !texts.is_empty())?;
        let mut next = Vec::with_capacity(combos.len() * alternatives.len());
        for combo in &combos {
            for text in alternatives {
                let mut extended = combo.clone();
                extended.push(text);
                next.push(extended);
            }
        }
        if next.len() > MAX_EMBEDDED_SQL_VARIANTS {
            return None;
        }
        combos = next;
    }
    let ranges = markers
        .iter()
        .map(|(range, _)| range.clone())
        .collect::<Vec<_>>();
    let (pieces, _) = split(caller, &ranges);
    // Which combination slot fills each marker, in text order.
    let slots: Vec<usize> = markers
        .iter()
        .map(|(_, site)| {
            sites
                .binary_search(site)
                .expect("every marker's site is collected")
        })
        .collect();
    combos
        .iter()
        .map(|combo| {
            let mut parts = Vec::with_capacity(pieces.len() * 2);
            let mut pieces = pieces.iter().cloned();
            parts.extend(pieces.next());
            for (slot, piece) in slots.iter().zip(pieces) {
                parts.push(combo[*slot].clone());
                parts.push(piece);
            }
            compose(parts)
        })
        .collect()
}
