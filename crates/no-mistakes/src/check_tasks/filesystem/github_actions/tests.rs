use super::CHECKS;
use std::collections::BTreeSet;

#[test]
fn workflow_checks_are_registered_once() {
    let mut registered = BTreeSet::new();
    for (rule, _) in CHECKS {
        assert!(registered.insert(rule), "duplicate workflow check {rule}");
    }
}
