mod fill;
use super::unpublish::{unpublish, unpublish_origins};
use crate::codebase::postgres::embedded::placeholders::publish_placeholders_with_positions;
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::postgres::embedded::walk::resolve::rewrite_origins;
use crate::codebase::postgres::embedded::{
    EmbeddedSqlCall, EmbeddedSqlKind, EmbeddedSqlVariant, MAX_EMBEDDED_SQL_VARIANTS,
};
use crate::fx::FxHashMap;
use fill::fill_all;

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
        if variants.len() > MAX_EMBEDDED_SQL_VARIANTS {
            return None;
        }
        let mut call = EmbeddedSqlCall {
            variants,
            ..template.clone()
        };
        // Finalization removes the raw mirror of a complete singleton so only
        // its provenance-bearing variant is spliced. Rebuild that public mirror
        // from the filled result, never from the marker-bearing template.
        if template.kind != EmbeddedSqlKind::Dynamic
            && template.variants.len() == 1
            && !template.variants_truncated()
        {
            if call.variants.len() == 1 && !call.variants_truncated() {
                let filled = &call.variants[0];
                call.sql_text = Some(filled.sql_text.clone());
                call.kind = EmbeddedSqlKind::Composed;
                call.declaration_line = Some(filled.line);
                call.sql_source_positions = filled.sql_source_positions.clone();
                call.recovered_placeholder_positions =
                    filled.recovered_placeholder_positions.clone();
            } else {
                call.sql_text = None;
                call.kind = EmbeddedSqlKind::Dynamic;
                call.declaration_line = None;
                call.sql_source_positions.clear();
                call.recovered_placeholder_positions.clear();
            }
        }
        return Some(call);
    }
    let published = template.sql_text.as_deref()?;
    let first_line = match template.kind {
        EmbeddedSqlKind::Inline => template.line,
        _ => template.declaration_line.unwrap_or(template.line),
    };
    let sql = unpublish(published, &template.recovered_placeholder_positions)?;
    let caller = RecoveredSql {
        invocations: Default::default(),
        origins: if template.sql_source_offsets.is_empty() {
            vec![0; sql.len()]
        } else {
            unpublish_origins(
                published,
                &template.recovered_placeholder_positions,
                &template.sql_source_offsets,
            )?
        },
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
    let sql_source_offsets = rewrite_origins(&joined.sql, &joined.origins, 0, true);
    let (sql_text, recovered_placeholder_positions) =
        publish_placeholders_with_positions(joined.sql);
    Some(EmbeddedSqlCall {
        sql_text: Some(sql_text),
        kind: EmbeddedSqlKind::Composed,
        declaration_line: Some(joined.line),
        sql_source_positions: joined.positions,
        sql_source_offsets,
        recovered_placeholder_positions,
        ..template.clone()
    })
}

fn splice_variant(variant: &EmbeddedSqlVariant, holes: &Holes) -> Option<Vec<EmbeddedSqlVariant>> {
    let positions = &variant.recovered_placeholder_positions;
    let caller = RecoveredSql {
        invocations: super::unpublish::unpublish_invocations(
            &variant.sql_text,
            &variant.recovered_placeholder_positions,
            variant.invocations.clone(),
        )?,
        sql: unpublish(&variant.sql_text, positions)?,
        origins: unpublish_origins(&variant.sql_text, positions, &variant.sql_source_offsets)?,
        line: variant.line,
        positions: variant.sql_source_positions.clone(),
    };
    Some(
        fill::fill_with_boundaries(&caller, holes, fill::internal_boundary(variant))?
            .into_iter()
            .map(|(joined, boundary)| {
                let mut filled = publish_variant(joined, variant.append_sites.clone());
                filled.variants_truncated = variant.variants_truncated;
                filled.first_append_offset = boundary;
                filled
            })
            .collect(),
    )
}

fn publish_variant(joined: RecoveredSql, append_sites: Vec<u32>) -> EmbeddedSqlVariant {
    let invocations = joined.invocations.publish(&joined.sql);
    let sql_source_offsets = rewrite_origins(&joined.sql, &joined.origins, 0, true);
    let (sql_text, recovered_placeholder_positions) =
        publish_placeholders_with_positions(joined.sql);
    EmbeddedSqlVariant {
        invocations,
        sql_text,
        sql_source_offsets,
        line: joined.line,
        sql_source_positions: joined.positions,
        recovered_placeholder_positions,
        append_sites,
        first_append_offset: None,
        variants_truncated: false,
    }
}
