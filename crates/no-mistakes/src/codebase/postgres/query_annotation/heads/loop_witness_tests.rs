#[test]
fn saved_loop_grammar_rejects_non_witnesses() {
    let (_, _, heads) = super::support::fixture("../loop-head-witness/query.mts");
    // Namespace trust is resolved by the consuming profile; the syntax
    // witness retains its imported name without trusting it here.
    assert_eq!(heads.loop_carried_missing.len(), 5);
    assert_eq!(
        heads
            .loop_carried_missing
            .values()
            .filter(|tag| tag.is_none())
            .count(),
        3
    );
}
