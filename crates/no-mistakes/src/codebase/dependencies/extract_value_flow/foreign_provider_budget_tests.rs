use super::*;
#[test]
fn missing_foreign_provider_roots_preserve_empty_results_without_spending_budget() {
    let history = ReceiverHistories {
        assigned_origins: Some(Default::default()),
        ..Default::default()
    };
    let mut work = mutable_origins::WorkBudget::default();
    let remaining = work.remaining();
    assert_eq!(
        history.foreign_member_provider_references(&(0, "missing".into()), &[], &mut work),
        Some(Vec::new())
    );
    assert_eq!(work.remaining(), remaining);
}
