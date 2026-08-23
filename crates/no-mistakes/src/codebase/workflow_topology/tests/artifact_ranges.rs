use super::super::artifact_pattern_match::matches_artifact_pattern;

#[test]
fn expands_a_reverse_alpha_range_and_rejects_invalid_alpha_steps() {
    assert!(matches_artifact_pattern("shard-b", "shard-{d..a}"));
    assert!(matches_artifact_pattern("shard-a", "shard-{a..g..2}"));
    assert!(!matches_artifact_pattern("shard-b", "shard-{a..g..2}"));
    assert!(!matches_artifact_pattern("shard-a", "shard-{a..g..x}"));
    assert!(!matches_artifact_pattern("shard-a", "shard-{aa..b}"));
}

#[test]
fn extglob_and_unbalanced_braces_are_conservative() {
    assert!(matches_artifact_pattern("anything", "logs-@(build|test)"));
    assert!(!matches_artifact_pattern("shard-a", "shard-{a"));
    assert!(matches_artifact_pattern("a{b}c", "a{b}c"));
}
