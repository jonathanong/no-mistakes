use super::*;

#[test]
fn call_intervals_fail_closed_for_missing_or_mixed_readers_and_isolate_epochs() {
    let mut effects = FreshReceiverEffects::default();
    let module = |offset| Some((offset, None));
    let callable = |offset| Some((offset, Some(CallableId(9))));
    assert!(effects.called_between(None, module(10)));
    assert!(!effects.called_between(module(1), module(10)));
    // A cold call does not create a proof index. Capturing a receiver enables it.
    effects.call(5, None);
    assert!(effects.calls.is_none());
    effects.capture(&[module(1)]);
    effects.call(5, None);
    assert!(effects.called_between(module(1), module(10)));
    assert!(!effects.called_between(module(5), module(10)));
    assert!(effects.called_between(module(1), callable(10)));
    assert!(effects.called_between(callable(1), module(10)));
    assert!(!effects.called_between(callable(1), callable(10)));
    effects.epoch = 1;
    assert!(!effects.called_between(module(1), module(10)));
    effects.call(7, Some(CallableId(9)));
    assert!(effects.called_between(callable(1), callable(10)));
    assert!(!effects.called_between(module(1), module(10)));
}

#[test]
fn root_rebinding_is_scoped_and_monotone() {
    let mut effects = FreshReceiverEffects::default();
    assert!(!effects.rebound(1, "holder"));
    assert!(effects.rebound.is_none());
    effects.rebind(1, "holder");
    effects.rebind(1, "holder");
    assert!(effects.rebound(1, "holder"));
    assert!(!effects.rebound(0, "holder"));
    assert!(!effects.rebound(1, "sibling"));
    assert_eq!(effects.rebound.as_ref().unwrap()[&1].len(), 1);
}
