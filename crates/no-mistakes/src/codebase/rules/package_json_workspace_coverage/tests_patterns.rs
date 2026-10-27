use super::super::patterns::unmatched_patterns;

#[test]
fn positive_patterns_match_alone_and_invalid_globs_fail() {
    let missing = unmatched_patterns(
        &[
            "packages/*".to_string(),
            "apps/*".to_string(),
            "!packages/hidden".to_string(),
        ],
        &["packages/hidden".to_string()],
    )
    .unwrap();
    assert_eq!(missing, vec!["apps/*".to_string()]);
    assert!(unmatched_patterns(&["[".to_string()], &[]).is_err());
}
