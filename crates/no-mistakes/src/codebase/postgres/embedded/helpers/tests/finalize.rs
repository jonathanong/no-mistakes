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
