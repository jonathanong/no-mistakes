use super::{activity_types_for, ACTIVITY_TYPES, ACTIVITY_TYPE_TRIGGERS};

#[test]
fn every_activity_trigger_has_one_nonempty_catalog() {
    assert_eq!(ACTIVITY_TYPES.len(), ACTIVITY_TYPE_TRIGGERS.len());
    for trigger in ACTIVITY_TYPE_TRIGGERS {
        assert!(!activity_types_for(trigger).is_empty(), "{trigger}");
    }
    assert!(activity_types_for("push").is_empty());
}
