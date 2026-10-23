//! A trimmed fragment must not keep placeholder positions from text that was cut.

use super::{coordinate, sentinel, INTERNAL_PLACEHOLDER};
use crate::codebase::postgres::embedded::helpers::finalize::{finalize, Walked};
use crate::codebase::postgres::embedded::relative::PendingRelativeCall;
use crate::codebase::postgres::embedded::{
    EmbeddedSqlCall, EmbeddedSqlFragment, EmbeddedSqlVariant,
};
use std::collections::BTreeMap;

fn finish(fragments: &mut [EmbeddedSqlFragment]) {
    let mut calls: [EmbeddedSqlCall; 0] = [];
    let mut pending_calls: [PendingRelativeCall; 0] = [];
    let mut variants: [Vec<EmbeddedSqlVariant>; 0] = [];
    let spans = BTreeMap::new();
    let _ = finalize(
        Walked {
            calls: &mut calls,
            call_spans: &[],
            pending_calls: &mut pending_calls,
            pending_spans: &spans,
            fragments,
            fragment_variants: &mut variants,
        },
        &[],
    );
}

#[test]
fn trimmed_fragment_drops_placeholder_positions_past_the_hole() {
    let hole = sentinel::encode(0);
    let text = format!("SELECT {INTERNAL_PLACEHOLDER}1 WHERE {hole} AND {INTERNAL_PLACEHOLDER}2");
    let cut = coordinate(&text, sentinel::prefix(&text).len());
    let before = coordinate(&text, text.find(INTERNAL_PLACEHOLDER).expect("bind"));
    let after = coordinate(
        &text,
        text.rfind(INTERNAL_PLACEHOLDER).expect("suffix bind"),
    );
    assert!(before < cut && after >= cut);
    let untouched = EmbeddedSqlFragment {
        line: 2,
        sql_text: Some("SELECT 1".into()),
        recovered_placeholder_positions: vec![(1, 8)],
    };
    let mut fragments = vec![
        EmbeddedSqlFragment {
            line: 4,
            sql_text: Some(text),
            recovered_placeholder_positions: vec![before, after],
        },
        EmbeddedSqlFragment {
            line: 6,
            sql_text: Some(format!("SET x = 1 {hole}")),
            recovered_placeholder_positions: vec![(1, 12)],
        },
        untouched.clone(),
    ];
    finish(&mut fragments);
    assert_eq!(
        fragments[0].sql_text.as_deref(),
        Some(format!("SELECT {INTERNAL_PLACEHOLDER}1 WHERE ").as_str())
    );
    assert_eq!(fragments[0].recovered_placeholder_positions, vec![before]);
    assert!(fragments[0]
        .recovered_placeholder_positions
        .iter()
        .all(|position| *position < cut));
    assert_eq!(fragments[1].sql_text, None);
    assert!(fragments[1].recovered_placeholder_positions.is_empty());
    assert_eq!(fragments[2], untouched);
}

#[test]
fn deterministic_singleton_helper_recovery_keeps_suffix_and_provenance() {
    use super::{extract, plain, Marker};
    use crate::codebase::postgres::embedded::helpers::splice::splice;
    let facts = extract("singleton-append.ts");
    assert!(facts.calls.iter().all(|call| {
        call.sql_text
            .as_deref()
            .is_none_or(|sql| !sentinel::contains(sql))
            && call
                .variants
                .iter()
                .all(|variant| !sentinel::contains(&variant.sql_text))
    }));
    let pending = facts.pending_imported.first().expect("singleton pending");
    assert_eq!(pending.template.variants.len(), 1);
    assert!(pending.template.sql_text.is_none());
    assert!(pending.template.sql_source_positions.is_empty());
    assert!(pending.template.recovered_placeholder_positions.is_empty());
    let helper = plain(
        format!("helper = {INTERNAL_PLACEHOLDER}1"),
        Marker {
            line: 30,
            path: Some("/helper.ts".into()),
            origin: 700,
        },
    );
    let mut holes: crate::fx::FxHashMap<_, _> = [(pending.sites[0].0, vec![helper.clone()])]
        .into_iter()
        .collect();
    let filled = splice(&pending.template, &holes).expect("filled singleton");
    assert_eq!(filled.kind, super::EmbeddedSqlKind::Composed);
    let [variant] = filled.variants.as_slice() else {
        panic!("singleton")
    };
    assert_eq!(
        variant.sql_text,
        "SELECT sql_placeholder_1 WHERE helper = sql_placeholder_2 AND suffix = sql_placeholder_3"
    );
    assert_eq!(filled.sql_text.as_deref(), Some(variant.sql_text.as_str()));
    assert_eq!(filled.declaration_line, Some(variant.line));
    assert_eq!(filled.sql_source_positions, variant.sql_source_positions);
    assert_eq!(
        filled.recovered_placeholder_positions,
        variant.recovered_placeholder_positions
    );
    assert_eq!(variant.recovered_placeholder_positions.len(), 3);
    assert!(!sentinel::contains(&variant.sql_text));
    let helper_at = variant.sql_text.find("helper").expect("helper");
    assert_eq!(variant.sql_source_offsets[helper_at], 700);
    assert!(variant
        .sql_source_positions
        .iter()
        .any(|position| position.source_path.as_deref()
            == Some(std::path::Path::new("/helper.ts"))
            && position.source_line == 30));
    let suffix_at = variant.sql_text.find("suffix").expect("suffix");
    assert_ne!(variant.sql_source_offsets[suffix_at], 700);
    assert_eq!(
        super::origin_at(
            &variant.sql_source_positions,
            variant.line,
            coordinate(&variant.sql_text, suffix_at)
        ),
        (9, None)
    );
    assert_eq!(variant.sql_source_offsets.len(), variant.sql_text.len());
    // A helper with multiple outcomes cannot publish one outcome as raw SQL.
    let mut alternative = helper;
    alternative.sql = "FALSE".into();
    alternative.origins = vec![700; alternative.sql.len()];
    alternative.positions.clear();
    holes
        .get_mut(&pending.sites[0].0)
        .expect("hole")
        .push(alternative);
    let fanout = splice(&pending.template, &holes).expect("finite fanout");
    assert_eq!(fanout.variants.len(), 2);
    assert_eq!(fanout.kind, super::EmbeddedSqlKind::Dynamic);
    assert!(fanout.sql_text.is_none());
    assert!(fanout.sql_source_positions.is_empty());
    assert!(fanout.recovered_placeholder_positions.is_empty());
    assert!(fanout.declaration_line.is_none());
    assert!(fanout
        .variants
        .iter()
        .all(|variant| !sentinel::contains(&variant.sql_text)));
}

#[test]
fn incomplete_or_mismatched_singleton_helper_mirrors_stay_unknown() {
    use super::extract;
    use crate::codebase::postgres::embedded::EmbeddedSqlKind;
    let facts = extract("singleton-append.ts");
    let pending = facts.pending_imported.first().expect("singleton pending");
    for invalid in 0..3 {
        let mut call = pending.template.clone();
        call.sql_text = Some(call.variants[0].sql_text.clone());
        match invalid {
            0 => call.kind = EmbeddedSqlKind::Dynamic,
            1 => call.variants[0].variants_truncated = true,
            _ => call
                .sql_text
                .as_mut()
                .expect("raw mirror")
                .push_str(" drift"),
        }
        let mut calls = [call];
        let mut relative = [];
        let mut fragments = [];
        let mut variants = [];
        let sites = vec![pending.sites[0].1.clone()];
        let recovered = finalize(
            Walked {
                calls: &mut calls,
                call_spans: &[pending.span],
                pending_calls: &mut relative,
                pending_spans: &BTreeMap::new(),
                fragments: &mut fragments,
                fragment_variants: &mut variants,
            },
            &sites,
        );
        assert!(recovered.is_empty(), "invalid shape {invalid}");
        assert_eq!(calls[0].kind, EmbeddedSqlKind::Dynamic);
        assert!(calls[0].variants.is_empty());
        assert!(calls[0]
            .sql_text
            .as_deref()
            .is_none_or(|sql| !sentinel::contains(sql)));
    }
}
