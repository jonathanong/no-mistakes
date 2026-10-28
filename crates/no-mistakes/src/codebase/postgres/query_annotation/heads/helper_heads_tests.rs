use super::support::{assert_markers, fixture};

#[test]
fn local_fixed_return_helpers_use_only_lexically_owned_literal_heads() {
    let (source, _, heads) = fixture("helper-heads.mts");
    assert_eq!(assert_markers(&source, &heads), 16);
    let missing = source.find("read(missing())").unwrap() as u32;
    assert!(heads
        .calls
        .iter()
        .any(|(span, proof)| span.0 == missing && *proof == super::super::HeadProof::Missing));
    let opaque_missing = source.find("read(opaqueMissing(").unwrap() as u32;
    let custom_missing = source.find("read(customOpaqueMissing(").unwrap() as u32;
    assert!(heads.calls.iter().any(
        |(span, proof)| span.0 == opaque_missing && *proof == super::super::HeadProof::Missing
    ));
    assert!(heads.calls.iter().any(
        |(span, proof)| span.0 == custom_missing && *proof == super::super::HeadProof::Unproven
    ));
}
