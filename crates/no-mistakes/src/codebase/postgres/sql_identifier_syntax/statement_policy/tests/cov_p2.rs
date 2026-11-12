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

#[test]
fn cov_p2_incomplete_prepared_policy_inputs_fail_consistently_without_reading_sources() {
    for missing_source in [true, false] {
        let (mut facts, _) = prepare(false);
        let path = root().join("src/query.mts");
        let file = facts.ts.get_mut(&path).unwrap();
        if missing_source {
            file.source = None;
        } else {
            file.sql_identifier_syntax.profiles.clear();
        }
        let expected = if missing_source { "source" } else { "syntax" };
        for _ in 0..2 {
            let error = projection(&facts, &path, &profile(), EXPLICIT, 0)
                .err()
                .unwrap()
                .to_string();
            assert_eq!(
                error,
                format!("{EXPLICIT}: prepared low SQL {expected} missing")
            );
        }
    }
}
