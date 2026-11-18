use super::*;

#[test]
fn blocking_wrapper_config_round_trips_merges_and_preserves_legacy_lists() {
    let root = fixture("wait-forms");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let json = serde_json::to_value(&config).unwrap();
    assert_eq!(
        json["queues"]["factories"]["enqueueAndWait"],
        serde_json::json!(["submitForResult"])
    );
    let decoded: crate::config::v2::NoMistakesConfig = serde_json::from_value(json).unwrap();
    let other = crate::config::v2::load_v2_config(&root, Some(&root.join("merge.yml"))).unwrap();
    let mut roles = decoded.queues.factories;
    roles.merge(&other.queues.factories).unwrap();
    assert_eq!(roles.enqueue_and_wait, ["awaitResult", "submitForResult"]);

    let legacy =
        crate::config::v2::load_v2_config(&root, Some(&root.join("unconfigured.yml"))).unwrap();
    let json = serde_json::to_value(legacy).unwrap();
    assert_eq!(json["queues"]["factories"], serde_json::json!([]));
}

#[test]
fn escaped_wait_candidates_stay_inside_the_configured_wait_scope() {
    use crate::codebase::check_facts::{configure_family_path_scopes, FamilyPathScope};
    use crate::codebase::ts_resolver::normalize_path;
    let root = fixture("wait-forms");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    assert!(files
        .iter()
        .any(|path| path.ends_with("api/unrelated-escape.ts")));
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut plan = crate::codebase::check_facts::CheckFactPlan {
        queue: true,
        queue_factory_roles: config
            .queues
            .factories
            .clone()
            .with_project_wait_scopes(&config),
        ..Default::default()
    };
    configure_family_path_scopes(&root, &files, &config, &sources, &mut plan).unwrap();
    let FamilyPathScope::Only(paths) = plan.queue_paths else {
        panic!("wait candidates unexpectedly widened to the whole repository");
    };
    assert!(paths.contains(&normalize_path(&root.join("api/routes/escaped.ts"))));
    assert!(!paths.contains(&normalize_path(&root.join("api/unrelated-escape.ts"))));
}
