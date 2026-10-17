//! What the walk records for calls built from imported helpers.

use super::*;
use crate::codebase::postgres::embedded::helpers::PendingImportedSql;
use crate::codebase::postgres::EmbeddedSqlCall;

fn pending_for(facts: &EmbeddedSqlFileFacts, index: usize) -> Option<&PendingImportedSql> {
    let span = facts.call_spans[index];
    facts
        .pending_imported
        .iter()
        .find(|pending| pending.span == span)
}

fn call(facts: &EmbeddedSqlFileFacts, index: usize) -> &EmbeddedSqlCall {
    &facts.calls[index]
}

#[test]
fn appended_helper_is_a_hole_between_the_callers_own_fragments() {
    let facts = extract("pending.ts");
    assert_eq!(facts.calls.len(), 14);
    // Public form: exactly what the walk produced before imports were modeled.
    assert_eq!(call(&facts, 0).kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(
        call(&facts, 0).sql_text.as_deref(),
        Some("SELECT l.id FROM links l WHERE l.account_id = sql_placeholder_1 AND ")
    );
    assert_eq!(call(&facts, 0).recovered_placeholder_positions, [(1, 47)]);
    let pending = pending_for(&facts, 0).expect("pending");
    assert_eq!(pending.template.kind, EmbeddedSqlKind::Composed);
    let template = pending.template.sql_text.as_deref().expect("template");
    let holes = sentinel::find_all(template);
    assert_eq!(holes.len(), 1);
    assert!(template[holes[0].0.end..].ends_with(" ORDER BY l.id"));
    assert_eq!(
        pending.sites,
        [(
            holes[0].1,
            ImportedSite {
                local: "ownerVisible".into(),
                args: vec![SiteArg::Literal("l.parent_id".into())],
                line: 8,
            }
        )]
    );
}

#[test]
fn an_insert_prefix_survives_for_the_insert_fail_closed_policy() {
    let facts = extract("pending.ts");
    assert_eq!(call(&facts, 1).kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(
        call(&facts, 1).sql_text.as_deref(),
        Some("INSERT INTO links (id) VALUES (1) ")
    );
    assert!(pending_for(&facts, 1).is_some());
}

#[test]
fn an_executor_argument_can_be_the_helper_call_itself() {
    let facts = extract("pending.ts");
    assert_eq!(call(&facts, 2).sql_text, None);
    let pending = pending_for(&facts, 2).expect("pending");
    assert_eq!(
        pending.template.sql_text,
        Some(sentinel::encode(pending.sites[0].0))
    );
}

#[test]
fn argument_kinds_are_recorded_for_the_projection() {
    let facts = extract("pending.ts");
    let args = |index| pending_for(&facts, index).map(|pending| pending.sites[0].1.args.clone());
    assert_eq!(args(3), Some(vec![SiteArg::Scalar]));
    assert_eq!(args(5), Some(vec![SiteArg::Builder, SiteArg::Scalar]));
    let template = pending_for(&facts, 5)
        .and_then(|pending| pending.template.sql_text.clone())
        .expect("template");
    assert!(template.starts_with("SELECT 1 FROM p WHERE true"));
    assert!(sentinel::contains(&template));
}

#[test]
fn unexecuted_or_interpolated_helpers_never_become_holes() {
    let facts = extract("pending.ts");
    // A conditional append is not composed, so there is nothing to complete.
    assert!(pending_for(&facts, 4).is_none());
    assert_eq!(call(&facts, 4).sql_text.as_deref(), Some("SELECT 1 WHERE "));
    // An interpolated call is a bound value, not SQL text.
    assert!(pending_for(&facts, 6).is_none());
    assert_eq!(facts.pending_imported.len(), 7);
}

#[test]
fn argument_shapes_decide_whether_a_hole_is_recorded() {
    let facts = extract("pending.ts");
    let args = pending_for(&facts, 10).map(|pending| pending.sites[0].1.args.clone());
    assert_eq!(
        args,
        Some(vec![
            SiteArg::Literal("l.template".into()),
            SiteArg::Scalar,
            SiteArg::Scalar,
            SiteArg::Scalar,
        ])
    );
    // Spread, object and two-builder arguments cannot be reasoned about.
    for index in [11, 12, 13] {
        assert!(pending_for(&facts, index).is_none(), "{index}");
        assert_eq!(
            call(&facts, index).sql_text.as_deref(),
            Some("SELECT 1 WHERE ")
        );
    }
}

#[test]
fn aliased_or_alternative_helper_results_are_never_completed() {
    let facts = extract("pending.ts");
    // A stored helper result aliases a mutable builder.
    assert!(pending_for(&facts, 7).is_none());
    // Alternatives that include a hole publish no partial version set.
    assert!(pending_for(&facts, 9).is_none());
    assert!(call(&facts, 9).variants.is_empty());
    assert_eq!(call(&facts, 9).sql_text, None);
}

#[test]
fn a_builder_fragment_that_held_a_hole_keeps_only_its_leading_statement() {
    let facts = extract("pending.ts");
    assert_eq!(
        call(&facts, 8).sql_text.as_deref(),
        Some("SELECT 2 SELECT 1 WHERE ")
    );
    assert!(facts
        .fragments
        .iter()
        .any(|fragment| fragment.sql_text.as_deref() == Some("SELECT 1 WHERE ")));
}

#[test]
fn a_trimmed_fragment_publishes_only_the_binds_that_remain() {
    let facts = extract("pending.ts");
    let trimmed = facts
        .fragments
        .iter()
        .find(|fragment| fragment.sql_text.as_deref() == Some("SELECT sql_placeholder_1 WHERE "))
        .expect("trimmed fragment");
    assert_eq!(trimmed.recovered_placeholder_positions, [(1, 8)]);
}

#[test]
fn the_value_of_an_arbitrary_imported_call_is_not_a_sql_builder() {
    let facts = extract("pending.ts");
    let source = std::fs::read_to_string(fixture("pending.ts")).expect("fixture");
    let line = source
        .lines()
        .position(|line| line.contains("element.append"))
        .expect("append") as u32
        + 1;
    // Were `element` treated as a builder, this append would be a recorded fragment.
    assert!(facts.fragments.iter().all(|fragment| fragment.line != line));
}

#[test]
fn only_value_imports_can_supply_a_hole() {
    let facts = extract("imports.ts");
    let locals = (0..facts.calls.len())
        .map(|index| pending_for(&facts, index).map(|pending| pending.sites[0].1.local.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        locals,
        [
            None,
            None,
            Some("value"),
            Some("alias"),
            None,
            Some("defaultHelper"),
            None
        ]
    );
}
