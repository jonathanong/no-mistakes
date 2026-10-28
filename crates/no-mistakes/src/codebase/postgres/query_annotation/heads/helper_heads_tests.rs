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

#[test]
fn concise_arrow_heads_keep_tag_and_callable_identity_boundaries() {
    let (source, _, heads) = fixture("../helper-arrow-heads/src/query.mts");
    assert_eq!(assert_markers(&source, &heads), 18);
    for call in [
        "read(missing())",
        "read(opaqueMissing(dynamic))",
        "read(literalMissing())",
    ] {
        let start = source.find(call).unwrap() as u32;
        assert!(heads
            .calls
            .iter()
            .any(|(span, proof)| span.0 == start && *proof == super::super::HeadProof::Missing));
    }
}

#[test]
fn consumed_opaque_append_does_not_restore_an_untrusted_builder_head() {
    let (source, _, heads) = super::support::fixture_with_append_candidates(
        "../helper-arrow-heads/src/append.mts",
        true,
    );
    assert_eq!(assert_markers(&source, &heads), 7);
}

#[test]
fn one_hop_const_helper_export_aliases_keep_the_same_head_as_direct_exports() {
    let (_, _, heads) = fixture("../helper-arrow-heads/src/helpers.mts");
    for name in ["direct", "blockAlias", "conciseAlias"] {
        assert_eq!(
            heads.exported_helpers.get(name),
            Some(&super::super::HeadProof::Literal),
            "{name}"
        );
    }
}
