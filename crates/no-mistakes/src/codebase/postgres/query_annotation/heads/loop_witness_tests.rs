#[test]
fn saved_loop_grammar_rejects_non_witnesses() {
    let (_, _, heads) = super::support::fixture("../loop-head-witness/query.mts");
    // Namespace trust is resolved by the consuming profile; the syntax
    // witness retains its imported name without trusting it here.
    assert_eq!(heads.loop_carried_missing.len(), 6);
    assert_eq!(
        heads
            .loop_carried_missing
            .values()
            .filter(|tags| tags.iter().any(Option::is_none))
            .count(),
        4
    );
}

#[test]
fn saved_headers_and_statement_sequences_preserve_only_backedge_witnesses() {
    let (source, _, heads) = super::support::fixture("../loop-head-witness/headers.mts");
    let lines = heads
        .loop_carried_missing
        .keys()
        .map(|span| {
            crate::codebase::ts_source::byte_offset_to_line(&source, span.0 as usize) as usize
        })
        .collect::<Vec<_>>();
    let expected = source
        .lines()
        .enumerate()
        .flat_map(|(index, line)| {
            let count = if line.ends_with("// witness") {
                1
            } else if line.ends_with("// two witnesses") {
                2
            } else {
                0
            };
            std::iter::repeat_n(index + 1, count)
        })
        .collect::<Vec<_>>();
    assert_eq!(lines, expected);
}
