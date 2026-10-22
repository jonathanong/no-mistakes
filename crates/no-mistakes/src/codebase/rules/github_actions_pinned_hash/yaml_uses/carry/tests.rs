use super::Carry;

#[test]
fn a_shallower_uses_does_not_join_a_flow_sequence_anchor() {
    // Closing the flow clears the anchor. This guard still refuses a uses key
    // that is already outside the anchored sequence.
    let mut carry = Carry {
        flow_sequence_anchor: true,
        anchor_name: "shared".to_string(),
        anchor_depth: 2,
        flow_depth: 1,
        ..Default::default()
    };
    carry.bump_anchor_uses();
    assert!(carry.anchor_use_counts.is_empty());
}
