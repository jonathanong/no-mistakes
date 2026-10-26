use super::*;

#[test]
fn low_tag_heads_are_identical_across_saved_profiles() {
    let source = std::fs::read_to_string(root().join("src/query.mts")).unwrap();
    let expected = marked_lines(&source, "EXPECT_REPORT_");
    // These explicit oracles retain unsupported heads while accepting every tag identity.
    assert_eq!(
        expected,
        vec![13, 15, 16, 17, 19, 21, 22, 42, 43, 46, 48, 53]
    );
    assert_eq!(
        marked_lines(&source, "EXPECT_ACCEPTED_TAG"),
        vec![14, 32, 33, 35, 37, 38, 39, 40, 41, 50]
    );
    for config in [
        ".no-mistakes-report.yml",
        ".no-mistakes-tags-none.yml",
        ".no-mistakes-tags-other.yml",
    ] {
        let result = run(config);
        assert_eq!(
            call_lines(&result.rules, "src/query.mts"),
            expected,
            "{config}"
        );
        assert!(result.advisories.is_empty(), "{config}");
    }
    let mixed = run(".no-mistakes-tags-mixed.yml");
    let mut mixed_lines = call_lines(&mixed.rules, "src/query.mts");
    mixed_lines.dedup();
    assert_eq!(mixed_lines, expected);
    assert_eq!(
        call_lines(&mixed.advisories, "src/query.mts"),
        expected
            .into_iter()
            .filter(|line| *line != 43)
            .collect::<Vec<_>>()
    );
    // Both report and advisory profiles retain the same call-site suppressions.
    for line in marked_lines(&source, "EXPECT_LINE_SUPPRESSED") {
        assert!(!call_lines(&mixed.rules, "src/query.mts").contains(&line));
        assert!(!call_lines(&mixed.advisories, "src/query.mts").contains(&line));
    }
}
