use super::*;

#[test]
fn low_tag_acceptance_uses_each_saved_profile_without_inferred_builtin_trust() {
    let source = std::fs::read_to_string(root().join("src/query.mts")).unwrap();
    let configured = run(".no-mistakes-report.yml");
    let without_tags = run(".no-mistakes-tags-none.yml");
    let other = run(".no-mistakes-tags-other.yml");
    let mut no_tags = marked_lines(&source, "EXPECT_REPORT_");
    no_tags.extend(marked_lines(&source, "EXPECT_ACCEPTED_TAG"));
    no_tags.sort_unstable();
    assert_eq!(call_lines(&without_tags.rules, "src/query.mts"), no_tags);
    let mut other_tags = no_tags;
    other_tags.retain(|line| !marked_lines(&source, "EXPECT_OTHER_TAG_ACCEPTED").contains(line));
    assert_eq!(call_lines(&other.rules, "src/query.mts"), other_tags);
    let mixed = run(".no-mistakes-tags-mixed.yml");
    assert_eq!(call_lines(&mixed.advisories, "src/query.mts"), other_tags);
    let configured_lines = call_lines(&configured.rules, "src/query.mts");
    assert_eq!(call_lines(&mixed.rules, "src/query.mts"), configured_lines);
    assert_eq!(configured_lines, marked_lines(&source, "EXPECT_REPORT_"));
    for line in marked_lines(&source, "EXPECT_ACCEPTED_TAG") {
        assert!(!configured_lines.contains(&line));
    }
    assert!(other.advisories.is_empty());
    assert!(without_tags.advisories.is_empty());
}
