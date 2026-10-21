use super::*;
use crate::codebase::check_facts::CheckFactPlan;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("postgres-require-query-annotation")
            .join("workspace-importer-selection"),
    )
}

fn inputs() -> (PathBuf, NoMistakesConfig, Vec<PathBuf>) {
    let root = fixture();
    let config =
        serde_yaml::from_str(&std::fs::read_to_string(root.join("selection.yml")).unwrap())
            .unwrap();
    let inventory: Vec<String> =
        serde_json::from_str(&std::fs::read_to_string(root.join("inventory.json")).unwrap())
            .unwrap();
    let files = inventory.into_iter().map(|path| root.join(path)).collect();
    (root, config, files)
}

#[test]
fn prepared_scope_intersects_application_project_skip_rule_and_option_filters() {
    let (root, config, files) = inputs();
    let mut plan = CheckFactPlan::default();
    configure_prepared_postgres_plan(&config, &mut plan).unwrap();
    assert!(plan
        .query_annotation_workspace_importers
        .as_ref()
        .unwrap()
        .is_empty());
    configure_prepared_postgres_plan_with_scope(&root, &config, &files, &mut plan).unwrap();
    let selected = plan.query_annotation_workspace_importers.as_ref().unwrap();
    let expected = crate::fx::PathSet::from_iter(
        [
            root.join("packages/app/src/selected.mts"),
            root.join("repo/selected.mts"),
            // Explicit project roots reopen a globally skipped parent directory.
            root.join("generated/reopened/selected.mts"),
        ]
        .into_iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(&path)),
    );
    assert_eq!(selected.as_ref(), &expected);
    let first = Arc::clone(selected);
    configure_prepared_postgres_plan_with_scope(&root, &config, &files, &mut plan).unwrap();
    assert_eq!(
        plan.query_annotation_workspace_importers.as_deref(),
        Some(first.as_ref())
    );
}

#[test]
fn explicit_direct_scope_remains_unrestricted_and_configured_empty_is_closed() {
    let (root, config, files) = inputs();
    let mut direct = CheckFactPlan {
        query_annotation: true,
        ..Default::default()
    };
    configure_prepared_postgres_plan(&config, &mut direct).unwrap();
    configure_prepared_postgres_plan_with_scope(&root, &config, &files, &mut direct).unwrap();
    assert!(direct.query_annotation_workspace_importers.is_none());
    let mut empty = CheckFactPlan::default();
    configure_prepared_postgres_plan(&config, &mut empty).unwrap();
    configure_prepared_postgres_plan_with_scope(&root, &config, &[], &mut empty).unwrap();
    assert!(empty
        .query_annotation_workspace_importers
        .unwrap()
        .is_empty());
}

#[test]
fn invalid_option_glob_is_reported_without_publishing_partial_scope() {
    let root = fixture();
    let config =
        serde_yaml::from_str(&std::fs::read_to_string(root.join("invalid.yml")).unwrap()).unwrap();
    // Exercise the helper independently: configure normally validates these
    // option globs earlier, before the shared fact collection boundary.
    let mut plan = CheckFactPlan {
        query_annotation: true,
        query_annotation_workspace_importers: Some(Arc::default()),
        ..Default::default()
    };
    let error = configure_prepared_postgres_plan_with_scope(&root, &config, &[], &mut plan)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("postgres-require-query-annotation include"),
        "{error}"
    );
    assert!(plan
        .query_annotation_workspace_importers
        .unwrap()
        .is_empty());
}

#[test]
fn structural_rules_alone_do_not_create_annotation_workspace_demand() {
    let (root, mut config, files) = inputs();
    config
        .rules
        .retain(|rule| rule.rule == "postgres-lock-ordering");
    let mut plan = CheckFactPlan::default();
    configure_prepared_postgres_plan(&config, &mut plan).unwrap();
    configure_prepared_postgres_plan_with_scope(&root, &config, &files, &mut plan).unwrap();
    assert!(!plan.query_annotation);
    assert!(plan.query_annotation_workspace_importers.is_none());
}

#[test]
fn scope_projection_reuses_each_parsed_application_mask_before_profile_deduplication() {
    let (_, config, _) = inputs();
    let mut masks = Vec::new();
    let (_, _, grouped) = configured_with_annotation_masks(
        &config,
        &["postgres-require-query-annotation"],
        |rule, paths| {
            masks.push((rule, paths));
        },
    )
    .unwrap();
    assert_eq!(masks.len(), 3);
    assert_eq!(
        grouped.len(),
        2,
        "distinct applications share an executor profile"
    );
    for (_, mask) in masks {
        assert!(grouped
            .values()
            .flatten()
            .any(|scan| Arc::ptr_eq(&mask, &scan.paths)));
    }
}

#[test]
fn complete_plan_configuration_errors_precede_new_scope_selection_errors() {
    let root = fixture();
    let config =
        serde_yaml::from_str(&std::fs::read_to_string(root.join("precedence.yml")).unwrap())
            .unwrap();
    let old = configure_prepared_postgres_plan(&config, &mut CheckFactPlan::default())
        .unwrap_err()
        .to_string();
    let new = configure_prepared_postgres_plan_with_scope(
        &root,
        &config,
        &[],
        &mut CheckFactPlan::default(),
    )
    .unwrap_err()
    .to_string();
    assert_eq!(new, old);
    assert!(new.contains("postgres-required-predicates"), "{new}");
}

#[path = "invalid_scope_tests.rs"]
mod invalid_scope;
