use super::support::{assert_markers, fixture};

#[test]
fn fresh_builder_result_bindings_require_exact_safe_uses() {
    let (source, _, heads) = fixture("helper-bindings.mts");
    assert_eq!(assert_markers(&source, &heads), 21);
    for needle in ["read(b) //", "read(missing().append"] {
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
