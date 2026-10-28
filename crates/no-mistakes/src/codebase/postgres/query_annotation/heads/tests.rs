#[path = "append_tests.rs"]
mod append_tests;
#[path = "failures.rs"]
mod failures;
#[path = "gap_tests.rs"]
mod gap_tests;
#[path = "helper_fallback_tests.rs"]
mod helper_fallback_tests;
#[path = "helper_heads_tests.rs"]
mod helper_heads_tests;
#[path = "missing_append_tests.rs"]
mod missing_append_tests;
#[path = "missing_tests.rs"]
mod missing_tests;
#[path = "support.rs"]
mod support;
#[path = "tags_tests.rs"]
mod tags_tests;
use support::{assert_markers, fixture};
#[path = "assignment_gap_tests.rs"]
mod assignment_gap_tests;

#[test]
fn original_ast_heads_accept_only_the_two_low_effort_shapes() {
    let (source, ts, heads) = fixture("root.mts");
    assert!(assert_markers(&source, &heads) >= 50);
    assert!(ts
        .module_bindings
        .as_ref()
        .unwrap()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "Merged binding declarations are unsupported"));
}
#[test]
fn direct_only_input_needs_no_lexical_const_index() {
    let (source, _, heads) = fixture("direct-only.mts");
    assert_eq!(assert_markers(&source, &heads), 2);
}
#[test]
fn variant_count_cannot_change_the_original_head_shape() {
    let (source, _, heads) = fixture("variants.mts");
    assert_eq!(assert_markers(&source, &heads), 4);
}
#[test]
fn dynamic_use_gaps_reject_const_without_poisoning_direct_literals() {
    let (source, _, heads) = fixture("dynamic.cjs");
    assert_eq!(assert_markers(&source, &heads), 3);
}
#[test]
fn canonical_direct_eval_gap_rejects_const_beyond_its_call_span() {
    let (source, ts, heads) = fixture("eval.cjs");
    // This discriminator deliberately matches the existing canonical producer;
    // an eval call's own span does not cover the later lexical reference.
    assert!(ts
        .module_bindings
        .as_ref()
        .unwrap()
        .diagnostics
        .iter()
        .any(|diagnostic| {
            diagnostic.kind == "unsupported"
                && diagnostic.message == "Dynamic eval may change binding semantics"
        }));
    assert_eq!(assert_markers(&source, &heads), 3);
    let (source, ts, heads) = fixture("shadowed-eval.cjs");
    assert!(!ts
        .module_bindings
        .as_ref()
        .unwrap()
        .diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.message == "Dynamic eval may change binding semantics" }));
    assert_eq!(assert_markers(&source, &heads), 1);
}

#[path = "loop_witness_tests.rs"]
mod loop_witness_tests;

use super::{AnnotationHeads, Span};
use crate::codebase::ts_source::facts::TsFileFacts;
use oxc_ast::ast::Program;
pub(super) fn collect_with_helpers(
    program: &Program<'_>,
    ts: &TsFileFacts,
    executor_calls: impl Iterator<Item = Span>,
    candidates: &crate::fx::FxHashSet<Span>,
    include_all_calls: bool,
    helpers: bool,
) -> AnnotationHeads {
    super::collect_prepared(
        program,
        ts,
        super::HeadCallSpans {
            confirmed: executor_calls.collect(),
            pending: Default::default(),
        },
        candidates,
        include_all_calls,
        helpers,
        None,
    )
}
