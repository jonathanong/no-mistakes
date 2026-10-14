use super::*;

#[test]
fn certain_execution_does_not_make_an_original_receiver_fresh() {
    let mut events = ReferenceReplacements::default();
    events.record(Some("client"), 10, true, true);
    assert!(events.replaced("client", 15));
    events.record(Some("client"), 20, false, true);
    assert!(!events.replaced("client", 25));
    assert!(
        events.replaced("client", 15),
        "later restoration cannot rewrite a saved acquisition"
    );
    events.record(Some("client"), 30, true, true);
    events.record(None, 40, false, true);
    assert!(events.replaced("client", 35));
    assert!(!events.replaced("client", 45));
    events.record(Some("client"), 50, true, false);
    assert!(
        !events.replaced("client", 55),
        "conditional freshness cannot establish identity"
    );
    events.record(Some("client"), 60, true, true);
    events.record(Some("client"), 70, true, false);
    assert!(
        events.replaced("client", 75),
        "conditional freshness preserves an established distinct receiver"
    );
}

#[test]
fn regional_proof_is_indexed_shared_and_canceled_at_acquisition() {
    let mut events = ReferenceReplacements::default();
    events.record(Some("client"), 5, true, false);
    assert!(
        events.regions.is_none(),
        "ordinary conditional events allocate no regional index"
    );
    events.record_region(10, "client", 20);
    assert!(!events.replaced("client", 25));
    assert!(events.replaced_in_region(10, "client", 25));
    assert!(!events.replaced_in_region(11, "client", 25));
    assert!(!events.replaced_in_region(10, "client", 15));
    events.record(Some("client"), 30, false, false);
    assert!(!events.replaced_in_region(10, "client", 35));
    assert!(events.replaced_in_region(10, "client", 25));
    let mut alias = ReferenceReplacements::default();
    alias.record_region(10, "client", 40);
    events.merge(alias);
    assert!(events.replaced_in_region(10, "client", 45));
    events.record(None, 50, false, false);
    assert!(!events.replaced_in_region(10, "client", 55));
    assert_eq!(events.regions.as_ref().unwrap().len(), 1);
    assert_eq!(events.regions.as_ref().unwrap()[&10]["client"].len(), 2);
    #[cfg(feature = "test-instrumentation")]
    {
        assert_eq!(events.region_record_attempts, 2);
        assert_eq!(events.region_queries.get(), 7);
        let snapshot = events.clone();
        assert_eq!(snapshot, events);
        let _: &dyn Sync = &events;
        let _: Box<dyn Send> = Box::new(snapshot.clone());
        assert!(events.replaced_in_region(10, "client", 45));
        assert_ne!(snapshot, events, "instrumentation clones own their counter");
    }
}
