use super::super::support::fixture_with_effects;
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings, captured_prefix, syntax, HeadProof,
};
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn closed_capture_never_publishes_a_head_before_import_obligations_resolve() {
    let (_, _, heads) = fixture_with_effects("../captured-import-effects/src/many.mts");
    assert_eq!(heads.conditional_effects.len(), 64);
    assert_eq!(heads.pending_effects.len(), 1);
    assert!(heads.conditional_effects.iter().all(|condition| {
        condition.required == [heads.pending_effects[0].span]
            && heads.calls.get(&condition.span) == Some(&HeadProof::Unproven)
    }));
}

#[test]
fn a_recovered_callee_write_rejects_the_otherwise_closed_captured_owner() {
    let (source, ts, _) = fixture_with_effects("../captured-import-effects/src/negative-cache.mts");
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("captured-import-effects/src/negative-cache.mts");
    let observer = InvocationObserver::new(true);
    crate::ast::with_program(&path, &source, |program, _| {
        let mut syntax = syntax::collect_with_helpers(program, true, true, true);
        let callee = source.find("append(q, tail)").unwrap() as u32;
        // A partial canonical write fact must invalidate this exact callee,
        // even when source import metadata and builder ownership were recovered.
        syntax
            .assignments
            .unsupported_writes
            .insert((callee, callee + 6));
        let calls = crate::codebase::postgres::extract_embedded_sql_from_program(
            &path,
            program,
            &source,
            &crate::codebase::postgres::EmbeddedSqlOptions::configured("@bench/db", &[]),
        );
        let executors = calls
            .call_spans
            .into_iter()
            .collect::<crate::fx::FxHashSet<_>>();
        let bindings = Bindings::new(&syntax, &ts, executors.iter().copied(), true);
        assert_eq!(bindings.effects().pending.len(), 2);
        let closed = with_observer(Some(observer.clone()), || {
            captured_prefix::proofs(
                &syntax,
                ts.module_bindings.as_deref().unwrap(),
                &bindings,
                &executors,
            )
        });
        assert!(closed.required.is_empty());
        assert!(closed.unconditional.is_empty());
    })
    .unwrap();
    let work = observer.snapshot().work;
    assert_eq!(
        work.get("postgres.annotation.captured_lifetime_censuses")
            .copied(),
        Some(2)
    );
    assert_eq!(
        work.get("postgres.annotation.captured_pending_callee_censuses")
            .copied(),
        Some(1)
    );
}
