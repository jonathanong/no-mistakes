use super::*;

#[test]
fn failed_scope_selection_does_not_publish_an_earlier_applications_importers() {
    let (root, _, files) = inputs();
    let config =
        serde_yaml::from_str(&std::fs::read_to_string(root.join("invalid-scope.yml")).unwrap())
            .unwrap();
    let mut plan = CheckFactPlan::default();
    let error = configure_prepared_postgres_plan_with_scope(&root, &config, &files, &mut plan)
        .unwrap_err()
        .to_string();
    assert!(error.contains("include"), "{error}");
    // The first application selected a real file, but the incomplete union
    // must never become the prepared request's permission set.
    assert!(plan
        .query_annotation_workspace_importers
        .unwrap()
        .is_empty());
}
