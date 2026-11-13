use super::*;

#[test]
fn cov_p2_unsupported_policy_preserves_an_explicit_error_for_a_registered_application() {
    let (facts, _) = prepare_mode(false, Policies::LegacyOnly);
    let path = root().join("src/query.mts");
    let error = projection(&facts, &path, &profile(), "postgres-no-offset", 0)
        .err()
        .unwrap()
        .to_string();
    assert_eq!(error, "unsupported low SQL policy postgres-no-offset");
}
