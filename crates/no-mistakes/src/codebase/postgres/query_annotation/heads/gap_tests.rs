use super::support::{assert_markers_for, decisions_for, fixture, recollect};
use crate::codebase::postgres::{EmbeddedSqlOptions, TrustedSqlTag};
use crate::codebase::ts_source::module_facts::{ModuleDiagnostic, ModuleSpan};
use std::sync::Arc;
const FIXTURE: &str = "diagnostic-gaps.mts";
fn options() -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured("@bench/db", &[]).with_trusted_sql_tags(&[TrustedSqlTag {
        module: "@bench/tags".into(),
        name: "sql".into(),
    }])
}
fn diagnostic(span: Option<ModuleSpan>, kind: &str, message: &str) -> ModuleDiagnostic {
    ModuleDiagnostic {
        kind: kind.into(),
        message: message.into(),
        span,
    }
}
#[test]
fn global_and_canonical_eval_gaps_reject_ownership_but_keep_literal_heads() {
    let (source, original, heads) = fixture(FIXTURE);
    assert_eq!(assert_markers_for(&source, &heads, &options()), 4);
    for (kind, message, span, expected) in [
        (
            "semanticError",
            "unlocated ownership failure",
            None,
            [false, false, false, true],
        ),
        (
            "unsupported",
            "Dynamic eval may change binding semantics",
            Some(ModuleSpan { start: 0, end: 1 }),
            [false, false, false, true],
        ),
        (
            "semanticError",
            "Dynamic eval may change binding semantics",
            Some(ModuleSpan { start: 0, end: 1 }),
            [true; 4],
        ),
        (
            "unsupported",
            "unrelated supported ownership gap",
            Some(ModuleSpan { start: 0, end: 1 }),
            [true; 4],
        ),
    ] {
        let mut ts = original.clone();
        Arc::make_mut(ts.module_bindings.as_mut().unwrap())
            .diagnostics
            .push(diagnostic(span, kind, message));
        let heads = recollect(FIXTURE, &ts);
        assert_eq!(
            decisions_for(&source, &heads, &options())
                .into_iter()
                .map(|(_, accepted)| accepted)
                .collect::<Vec<_>>(),
            expected,
            "{kind}: {message}"
        );
    }
}
#[test]
fn nested_and_touching_diagnostic_intervals_do_not_hide_a_real_use_gap() {
    let (source, mut ts, heads) = fixture(FIXTURE);
    assert_eq!(assert_markers_for(&source, &heads, &options()), 4);
    let module = Arc::make_mut(ts.module_bindings.as_mut().unwrap());
    let references = &module
        .bindings
        .iter()
        .find(|binding| binding.name == "text")
        .unwrap()
        .references;
    assert_eq!(references.len(), 1);
    let usage = references[0].span;
    // Without merging, the contained interval makes sorted endpoints decrease;
    // binary search can then skip the enclosing interval covering the real use.
    for (start, end) in [
        (usage.end, usage.end + 1),
        (usage.start - 2, usage.end),
        (usage.start - 1, usage.start),
        (0, 1),
    ] {
        module.diagnostics.push(diagnostic(
            Some(ModuleSpan { start, end }),
            "semanticError",
            "ownership interval",
        ));
    }
    let heads = recollect(FIXTURE, &ts);
    assert_eq!(
        decisions_for(&source, &heads, &options())
            .into_iter()
            .map(|(_, accepted)| accepted)
            .collect::<Vec<_>>(),
        [false, true, true, true]
    );
}
#[test]
fn half_open_diagnostic_boundaries_and_unrelated_gaps_preserve_const_proofs() {
    let (source, mut ts, _) = fixture(FIXTURE);
    let module = Arc::make_mut(ts.module_bindings.as_mut().unwrap());
    let usage = module
        .bindings
        .iter()
        .find(|binding| binding.name == "text")
        .unwrap()
        .references[0]
        .span;
    for (start, end) in [(usage.start - 1, usage.start), (usage.end, usage.end + 1)] {
        module.diagnostics.push(diagnostic(
            Some(ModuleSpan { start, end }),
            "semanticError",
            "adjacent gap",
        ));
    }
    let heads = recollect(FIXTURE, &ts);
    assert_eq!(assert_markers_for(&source, &heads, &options()), 4);
}
