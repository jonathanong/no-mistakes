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

#[test]
fn multiline_placeholder_shapes_keep_the_expression_line_and_suppression() {
    let root = fixture("review-followups");
    let path = root.join("src/interpolation-start.ts");
    let mut findings =
        check_with_files(&root, &config_yaml(BANNED), std::slice::from_ref(&path)).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [3]
    );
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty());
}

#[test]
fn empty_grouping_sets_remain_global_existence_probes() {
    let root = fixture("review-followups");
    let findings = check_with_files(
        &root,
        &config_yaml(BANNED),
        &[root.join("sql/empty-grouping.sql")],
    )
    .unwrap();
    assert_eq!(
        findings.iter().map(|f| f.line).collect::<Vec<_>>(),
        [2, 3, 6, 10],
        "{findings:#?}"
    );
}

#[test]
fn scalar_counts_that_can_return_no_rows_are_not_rewritten_as_exists() {
    let root = fixture("review-followups");
    let findings = check_with_files(
        &root,
        &config_yaml(BANNED),
        &[root.join("sql/count-may-return-no-rows.sql")],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}
