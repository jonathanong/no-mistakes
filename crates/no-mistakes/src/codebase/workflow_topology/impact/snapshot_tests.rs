use super::is_workflow_tree;

#[test]
fn descends_only_to_workflow_directories() {
    assert!(is_workflow_tree(".github"));
    assert!(is_workflow_tree(".github/workflows"));
    assert!(is_workflow_tree(".github/workflows/reusable"));
    assert!(!is_workflow_tree(".github/actions"));
    assert!(!is_workflow_tree(".github/ISSUE_TEMPLATE"));
}
