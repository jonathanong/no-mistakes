use super::super::super::coverage::NoFacts;
use super::*;

#[test]
fn missing_module_facts_do_not_invent_cleanup() {
    let mut lifecycle = Lifecycle::default();
    lifecycle.record(Path::new("/repo/missing.mts"), &NoFacts);
    assert!(!lifecycle.cleanup_all);
    assert!(lifecycle.cleanup_names.is_empty());
}
