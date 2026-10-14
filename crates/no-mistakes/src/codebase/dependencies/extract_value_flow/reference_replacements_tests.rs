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
