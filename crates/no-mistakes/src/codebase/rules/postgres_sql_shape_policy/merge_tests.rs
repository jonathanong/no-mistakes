use super::shape_tests::{config_yaml, fixture, BANNED};
use super::*;

#[test]
fn merge_predicates_and_action_queries_collect_shapes_once() {
    let root = fixture("review-followups");
    let file = root.join("sql/merge.sql");
    let findings = check_with_files(&root, &config_yaml(BANNED), &[file]).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.target.as_deref(), finding.line))
            .collect::<Vec<_>>(),
        [
            (Some("not-in-subquery"), 2),
            (Some("not-in-subquery"), 3),
            (Some("count-for-existence"), 4),
            (Some("not-in-subquery"), 5),
            (Some("not-in-subquery"), 6),
            (Some("not-in-subquery"), 8),
        ],
        "{findings:#?}"
    );
}

#[test]
fn merge_table_function_arguments_collect_count_shapes() {
    let root = fixture("review-followups");
    let findings = check_with_files(
        &root,
        &config_yaml(BANNED),
        &[root.join("sql/merge-table-function.sql")],
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.target.as_deref(), finding.line))
            .collect::<Vec<_>>(),
        [
            (Some("count-for-existence"), 2),
            (Some("count-for-existence"), 7),
        ],
        "{findings:#?}"
    );
}

#[test]
fn merge_action_expressions_collect_outer_shapes() {
    let root = fixture("review-followups");
    let findings = check_with_files(
        &root,
        &config_yaml(BANNED),
        &[root.join("sql/merge-action.sql")],
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.target.as_deref(), finding.line))
            .collect::<Vec<_>>(),
        [
            (Some("not-in-subquery"), 5),
            (Some("count-for-existence"), 10),
        ],
        "{findings:#?}"
    );
}

#[test]
fn merge_with_cte_preserves_query_scope_and_predicate_shapes() {
    let root = fixture("review-followups");
    let file = root.join("sql/merge-with.sql");
    let findings = check_with_files(&root, &config_yaml(BANNED), &[file]).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [1, 4],
        "{findings:#?}"
    );
}
