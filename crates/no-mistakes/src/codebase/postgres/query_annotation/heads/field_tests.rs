use super::support::{assert_markers, fixture};

#[test]
fn trusted_local_builder_text_reads_require_exact_safe_origin() {
    let (source, _, heads) = fixture("text-field.mts");
    assert_eq!(assert_markers(&source, &heads), 15);
    let start = source.find("function missing(").unwrap();
    let call = start + source[start..].find("read(query.text").unwrap();
    assert_eq!(
        heads
            .calls
            .iter()
            .find(|(span, _)| span.0 == call as u32)
            .map(|(_, proof)| *proof),
        Some(super::super::HeadProof::Missing),
    );
}

#[test]
fn opaque_text_fields_keep_missing_and_transaction_boundary_proofs_distinct() {
    let (source, _, heads) = fixture("text-field.mts");
    use super::super::HeadProof::{Literal, Missing, Unproven};
    for (needle, expected) in [
        ("read(named.text", Literal),
        ("read(missing.text", Missing),
        ("read(transaction.text", Unproven),
    ] {
        let start = source.find(needle).unwrap() as u32;
        assert_eq!(
            heads
                .calls
                .iter()
                .find(|(span, _)| span.0 == start)
                .map(|(_, proof)| *proof),
            Some(expected)
        );
    }
}
