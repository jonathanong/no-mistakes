use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn forbidden_calls_mocked_owner_json_matches_snapshot() {
    let root = rules_path("forbidden-calls", "mocked-owner", "fixture");
    assert_saved_fixture(
        "forbidden_calls_mocked_owner",
        &[("mocked_owner", root.clone(), root.join(".no-mistakes.yml"))],
    );
}
