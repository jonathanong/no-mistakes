use super::shape_tests::{config_yaml, fixture, BANNED};
use super::*;

#[test]
fn conflict_update_where_collects_shape_and_nested_query_facts() {
    let root = fixture("review-followups");
    let file = root.join("sql/on-conflict-where.sql");
    let findings = check_with_files(&root, &config_yaml(BANNED), &[file]).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.target.as_deref(), finding.line))
            .collect::<Vec<_>>(),
        [
            (Some("not-in-subquery"), 3),
            (Some("count-for-existence"), 5),
        ],
        "{findings:#?}"
    );
}

#[test]
fn nullable_count_is_skipped_to_preserve_null_semantics() {
    let root = fixture("review-followups");
    let file = root.join("sql/nullable-count.sql");
    let findings = check_with_files(
        &root,
        &config_yaml("bannedShapes: [count-for-existence]\n"),
        &[file],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}
