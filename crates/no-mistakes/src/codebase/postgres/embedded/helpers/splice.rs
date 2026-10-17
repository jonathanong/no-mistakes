use super::sentinel;
use super::split::{compose, split};
use super::unpublish::{unpublish, unpublish_origins};
use crate::codebase::postgres::embedded::placeholders::publish_placeholders_with_positions;
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::postgres::embedded::walk::resolve::rewrite_origins;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlKind, EmbeddedSqlVariant};
use crate::fx::FxHashMap;

type Holes = FxHashMap<usize, RecoveredSql>;

/// Fills every hole of `template` and republishes it. A call whose own text
/// holds holes becomes a complete `Composed` call; a call whose holes sit in
/// its complete alternatives keeps its kind and legacy text and gets filled
/// alternatives. `holes` maps a hole's site index to its text; any gap yields
/// `None` so the scrubbed `Dynamic` call stays in place.
pub(crate) fn splice(template: &EmbeddedSqlCall, holes: &Holes) -> Option<EmbeddedSqlCall> {
    if !template.variants.is_empty() {
        let variants = template
            .variants
            .iter()
            .map(|variant| splice_variant(variant, holes))
            .collect::<Option<Vec<_>>>()?;
        return Some(EmbeddedSqlCall {
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
    let joined = fill(&caller, holes)?;
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

fn splice_variant(variant: &EmbeddedSqlVariant, holes: &Holes) -> Option<EmbeddedSqlVariant> {
    let positions = &variant.recovered_placeholder_positions;
    let caller = RecoveredSql {
        sql: unpublish(&variant.sql_text, positions)?,
        origins: unpublish_origins(&variant.sql_text, positions, &variant.sql_source_offsets)?,
        line: variant.line,
        positions: variant.sql_source_positions.clone(),
    };
    let joined = fill(&caller, holes)?;
    let sql_source_offsets = rewrite_origins(&joined.sql, &joined.origins, 0, true);
    let (sql_text, recovered_placeholder_positions) =
        publish_placeholders_with_positions(joined.sql);
    Some(EmbeddedSqlVariant {
        sql_text,
        sql_source_offsets,
        line: joined.line,
        sql_source_positions: joined.positions,
        recovered_placeholder_positions,
        append_sites: variant.append_sites.clone(),
    })
}

/// `caller` with every hole replaced by its text.
fn fill(caller: &RecoveredSql, holes: &Holes) -> Option<RecoveredSql> {
    let markers = sentinel::find_all(&caller.sql);
    let ranges = markers
        .iter()
        .map(|(range, _)| range.clone())
        .collect::<Vec<_>>();
    let (pieces, _) = split(caller, &ranges);
    let mut parts = Vec::with_capacity(pieces.len() * 2);
    let mut pieces = pieces.into_iter();
    parts.extend(pieces.next());
    for ((_, site), piece) in markers.iter().zip(pieces) {
        parts.push(holes.get(site)?.clone());
        parts.push(piece);
    }
    compose(parts)
}
