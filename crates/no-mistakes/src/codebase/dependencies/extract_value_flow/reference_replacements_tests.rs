use super::*;

#[test]
fn certain_execution_does_not_make_an_original_receiver_fresh() {
    let mut events = ReferenceReplacements::default();
    events.record(Some("client"), 10, true, true);
    assert!(events.replacement_offset("client", 15).is_some());
    events.record(Some("client"), 20, false, true);
    assert!(events.replacement_offset("client", 25).is_none());
    assert!(
        events.replacement_offset("client", 15).is_some(),
        "later restoration cannot rewrite a saved acquisition"
    );
    events.record(Some("client"), 30, true, true);
    events.record(None, 40, false, true);
    assert!(events.replacement_offset("client", 35).is_some());
    assert!(events.replacement_offset("client", 45).is_none());
    events.record(Some("client"), 50, true, false);
    assert!(
        events.replacement_offset("client", 55).is_none(),
        "conditional freshness cannot establish identity"
    );
    events.record(Some("client"), 60, true, true);
    events.record(Some("client"), 70, true, false);
    assert!(
        events.replacement_offset("client", 75).is_some(),
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
    assert!(events.replacement_offset("client", 25).is_none());
    assert!(events.regional_offset(10, "client", 25).is_some());
    assert!(events.regional_offset(11, "client", 25).is_none());
    assert!(events.regional_offset(10, "client", 15).is_none());
    events.record(Some("client"), 30, false, false);
    assert!(events.regional_offset(10, "client", 35).is_none());
    assert!(events.regional_offset(10, "client", 25).is_some());
    let mut alias = ReferenceReplacements::default();
    alias.record_region(10, "client", 40);
    events.merge(alias);
    assert!(events.regional_offset(10, "client", 45).is_some());
    events.record(None, 50, false, false);
    assert!(events.regional_offset(10, "client", 55).is_none());
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
        assert!(events.regional_offset(10, "client", 45).is_some());
        assert_ne!(snapshot, events, "instrumentation clones own their counter");
    }
}
