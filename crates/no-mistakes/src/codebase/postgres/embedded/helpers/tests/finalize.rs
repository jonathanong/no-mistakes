//! A trimmed fragment must not keep placeholder positions from text that was cut.

use super::{INTERNAL_PLACEHOLDER, coordinate, sentinel};
use crate::codebase::postgres::embedded::helpers::finalize::{Walked, finalize};
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
        &crate::fx::FxHashMap::default(),
    );
}

fn fragment_site() -> crate::codebase::postgres::ImportedSite {
    use crate::codebase::postgres::{ImportedSite, SiteKind};
    ImportedSite {
        local: "scopeCondition".into(),
        member: None,
        kind: SiteKind::Sql,
        args: Vec::new(),
        value_used: true,
        line: 1,
        constant: Some(true),
    }
}

fn finalize_text(sql: String, sites: &[crate::codebase::postgres::ImportedSite]) -> Option<String> {
    use super::extract;
    use crate::codebase::postgres::embedded::EmbeddedSqlKind;
    let facts = extract("singleton-append.ts");
    let pending = facts.pending_imported.first().expect("singleton pending");
    let mut call = pending.template.clone();
    call.variants.clear();
    call.kind = EmbeddedSqlKind::Dynamic;
    call.sql_text = Some(sql);
    let mut calls = [call];
    let mut relative = [];
    let mut fragments = [];
    let mut variants = [];
    let _ = finalize(
        Walked {
            calls: &mut calls,
            call_spans: &[pending.span],
            pending_calls: &mut relative,
            pending_spans: &BTreeMap::new(),
            fragments: &mut fragments,
            fragment_variants: &mut variants,
        },
        sites,
        &crate::fx::FxHashMap::default(),
    );
    assert!(calls[0].variants.is_empty());
    assert!(!calls[0].sql_text.as_deref().is_some_and(sentinel::contains));
    calls[0].sql_text.clone()
}

#[path = "finalize_cases/part1.rs"]
mod part1;

#[path = "finalize_cases/part2.rs"]
mod part2;
