use super::support::fixture_with_effects;
use std::{path::PathBuf, sync::Arc};

#[test]
fn duplicate_prepared_import_identity_rejects_returned_parameter_projection() {
    let (source, mut ts, heads) = fixture_with_effects("returned-param-ambiguity.mts");
    assert_eq!(heads.pending_returned_params.len(), 1);
    let module = Arc::make_mut(ts.module_bindings.as_mut().unwrap());
    let import = module
        .imports
        .iter_mut()
        .find(|item| item.specifier == "./other.mjs")
        .unwrap();
    import.bindings.push(import.bindings[0].clone());
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/low-heads/returned-param-ambiguity.mts");
    let calls = heads
        .calls
        .keys()
        .copied()
        .filter(|span| source[span.0 as usize..].starts_with("read("))
        .collect::<Vec<_>>();
    let changed = crate::ast::with_program(&path, &source, |program, _| {
        super::collect_with_helpers(
            program,
            &ts,
            calls.into_iter(),
            &Default::default(),
            true,
            true,
        )
    })
    .unwrap();
    assert!(changed.pending_returned_params.is_empty());
}

#[test]
fn returned_parameter_roots_require_continuation_safe_heads() {
    use super::super::HeadProof;
    let (source, _, heads) = fixture_with_effects("returned-param-continuation.mts");
    assert_eq!(super::support::assert_markers(&source, &heads), 6);
    // Only stable roots enter the prepared source-hop queue. EOF BEGIN and
    // partial/empty comments must remain unproven regardless of helper identity.
    let proofs = heads
        .pending_returned_params
        .iter()
        .map(|p| p.proof)
        .collect::<Vec<_>>();
    assert_eq!(
        proofs,
        vec![HeadProof::Missing, HeadProof::Literal, HeadProof::Missing]
    );
}
