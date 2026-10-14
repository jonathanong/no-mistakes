use super::*;

#[test]
fn late_proof_demand_and_untracked_union_cannot_supply_missing_events() {
    let mut late = ReceiverHistories::default();
    let key = (0, "late".to_string());
    late.history_mut(&key)
        .0
        .record(&[Some("client".into())], 20, None);
    late.require_reference_proof(0, "late");
    assert!(!late.history(&key).unwrap().1);

    let mut linked = ReceiverHistories::default();
    let tracked = (0, "tracked".to_string());
    linked.require_reference_proof(0, "tracked");
    linked.history_mut(&tracked).0.record(&[], 10, None);
    assert!(linked.history(&tracked).unwrap().1);
    linked.link((0, "tracked"), (0, "empty"));
    assert!(linked.history(&tracked).unwrap().1);
    let unknown = (0, "unknown".to_string());
    linked.history_mut(&unknown).0.record(&[], 30, None);
    linked.link((0, "tracked"), (0, "unknown"));
    assert!(!linked.history(&tracked).unwrap().1);
    assert!(!linked.history(&unknown).unwrap().1);
}
