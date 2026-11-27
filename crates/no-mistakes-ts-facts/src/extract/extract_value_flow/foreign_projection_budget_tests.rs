use super::*;
fn reference() -> nested::ReceiverWrite {
    (
        ((0, "receiver".into()), vec![Some("member".into())]),
        vec![None],
        (0, None),
    )
}
fn output(remaining: Option<usize>) -> ProjectionOutput<nested::RootRead> {
    ProjectionOutput {
        output: Default::default(),
        remaining,
        used: 0,
        exceeded: false,
    }
}
#[test]
fn ordinary_projection_keeps_order_and_allocations_while_bounded_projection_stops_retaining() {
    let mut ordinary = output(None);
    ordinary.extend([reference(), reference()]);
    let (first, overflow) = ordinary.output.into_parts();
    assert_eq!(first, Some(reference()));
    assert_eq!(overflow, [reference()]);
    let mut bounded = output(Some(3));
    bounded.extend([reference(), reference(), reference()]);
    assert!(bounded.exceeded);
    assert_eq!(bounded.used, 3);
    assert_eq!(bounded.remaining, Some(0));
    let (first, overflow) = bounded.output.into_parts();
    assert_eq!(first, Some(reference()));
    assert!(overflow.is_empty());
}

#[test]
fn bounded_output_stops_consuming_input_after_the_first_rejected_reference() {
    let generated = std::cell::Cell::new(0);
    let mut bounded = output(Some(3));
    bounded.extend((0..1024).map(|_| {
        generated.set(generated.get() + 1);
        reference()
    }));
    assert!(bounded.exceeded);
    assert_eq!(generated.get(), 2);
}

#[test]
fn expression_visits_exhaust_the_budget_before_any_reference_is_emitted() {
    use nested::ReceiverOutput;
    let mut bounded = output(Some(1));
    assert!(bounded.visit());
    assert!(bounded.active());
    assert_eq!(bounded.used, 1);
    assert_eq!(bounded.remaining, Some(0));
    assert!(!bounded.visit());
    assert!(!bounded.active());
    assert!(bounded.exceeded);
    let (first, overflow) = bounded.output.into_parts();
    assert!(first.is_none());
    assert!(overflow.is_empty());
}
#[cfg(feature = "test-instrumentation")]
#[test]
fn exhausted_projection_is_discarded_before_pending_queue_append() {
    let mut collector = ImportCollector {
        foreign_receiver_writes: Some(Box::new(ForeignReceiverWrites::finishing_with(
            WorkBudget::default(),
        ))),
        ..Default::default()
    };
    let mut bounded = output(Some(3));
    bounded.extend([reference(), reference()]);
    let mut pending = PendingQueue::from_vec(Vec::new());
    assert!(!collector.append_bounded_mutation(
        &mut pending,
        0,
        (0, "receiver".into()),
        (0, None),
        false,
        bounded
    ));
    assert_eq!(pending.len(), 0);
    assert!(collector.foreign_work_exhausted());
}
