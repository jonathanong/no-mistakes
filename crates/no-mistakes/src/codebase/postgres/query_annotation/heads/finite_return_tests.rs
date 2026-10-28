use super::support::{assert_markers, fixture};

#[test]
fn explicit_total_returns_join_only_the_same_proven_head() {
    let (source, _, heads) = fixture("finite-helper-returns.mts");
    assert_eq!(assert_markers(&source, &heads), 21);
    for needle in ["read(missing(true))", "read(switchedMissing('a'))"] {
        let start = source.find(needle).unwrap() as u32;
        assert_eq!(
            heads
                .calls
                .iter()
                .find(|(span, _)| span.0 == start)
                .map(|(_, proof)| *proof),
            Some(super::super::HeadProof::Missing),
        );
    }
}

#[test]
fn finite_return_guards_do_not_hide_builder_mutations_or_returns() {
    let (source, _, heads) = fixture("finite-helper-guards.mts");
    assert_eq!(assert_markers(&source, &heads), 18);
}
