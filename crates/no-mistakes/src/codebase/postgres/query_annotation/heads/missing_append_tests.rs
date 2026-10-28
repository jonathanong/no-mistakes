use super::support::{fixture, fixture_with_append_candidates};

#[test]
fn append_preserves_missing_proofs_and_invalidates_eof_only_classifications() {
    let (source, _, heads) = fixture("append-missing.mts");
    let mut count = 0;
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        if let Some((before, expected)) = line.split_once("// head: ") {
            let start = offset + before.find("read(").unwrap();
            let (&span, _) = heads
                .calls
                .iter()
                .find(|(span, _)| span.0 as usize == start)
                .unwrap();
            let observed = if heads.accepts(span) {
                "clear"
            } else if heads.proves_missing(span) {
                "missing"
            } else {
                "unknown"
            };
            assert_eq!(observed, expected.trim(), "{line}");
            count += 1;
        }
        offset += line.len();
    }
    assert_eq!(count, 34);
}

#[test]
fn opaque_missing_head_requires_exact_trusted_tag_and_unescaped_local_builder() {
    let (source, _, heads) = fixture_with_append_candidates("opaque-missing-trust.mts", true);
    let actual = heads
        .calls
        .iter()
        .filter(|((start, _), _)| source[*start as usize..].starts_with("read("))
        .map(|(_, proof)| *proof)
        .collect::<Vec<_>>();
    use super::super::HeadProof::{Missing, Unproven};
    assert_eq!(
        actual,
        [Missing, Unproven, Unproven, Missing, Unproven, Unproven]
    );
}
