use super::*;
use std::cell::Cell;

#[test]
fn reference_sites_are_computed_only_for_enabled_materialized_pools() {
    for (enabled, registered) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut scope = Scope::default().with_bodies(enabled);
        scope.insert("pool".into(), SqlBoundQuery::default());
        if registered {
            scope.mark_pool("pool");
        }
        let calls = Cell::new(0);
        scope.record_pool_reference(scope.get("pool").unwrap(), || {
            calls.set(calls.get() + 1);
            (2, 1)
        });
        assert_eq!(calls.get(), usize::from(enabled && registered));
    }
}
