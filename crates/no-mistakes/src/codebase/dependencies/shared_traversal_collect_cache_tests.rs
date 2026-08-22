#[test]
fn traversal_cache_error_helpers_round_trip_the_message() {
    let cached = super::cache_traversal_error(anyhow::anyhow!("boom"));
    assert!(cached.contains("boom"));
    assert!(super::replay_cached_traversal_error(cached)
        .to_string()
        .contains("boom"));
}
