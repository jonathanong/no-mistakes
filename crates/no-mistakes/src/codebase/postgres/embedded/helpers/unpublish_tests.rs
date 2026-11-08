use super::{unpublish, unpublish_invocations};
use crate::codebase::postgres::embedded::site_identity::Invocations;

#[test]
fn generated_marker_widths_round_trip_invocation_boundaries() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-site-identity").join("identity.sql"),
    )
    .unwrap();
    let mut sites = Invocations::default();
    sites.record(sql.len(), 11);
    let positions = [(1, 8), (2, 8)];
    let raw = unpublish(&sql, &positions).unwrap();
    let restored = unpublish_invocations(&sql, &positions, sites.clone()).unwrap();
    assert_eq!(restored.publish(&raw), sites);
    assert!(unpublish_invocations(&sql, &[(99, 1)], sites).is_none());
    assert!(
        unpublish_invocations(&sql, &positions, Invocations::default())
            .unwrap()
            .is_empty()
    );
}
