use super::*;
use crate::config::v2::load_v2_config;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/wait-wrappers/fixture")
}

#[test]
fn patterns_round_trip_share_compilation_and_merge_policy() {
    let root = root();
    let config = load_v2_config(&root, Some(&root.join("patterns.yml"))).unwrap();
    let encoded = serde_json::to_value(&config).unwrap();
    assert!(encoded["queues"]["factories"]
        .get("forbidEnqueueAndWaitDeclarations")
        .is_none());
    let decoded: crate::config::v2::NoMistakesConfig = serde_json::from_value(encoded).unwrap();
    let mut roles = decoded.queues.factories;
    let copy = roles.clone();
    assert!(Arc::ptr_eq(
        &roles.enqueue_and_wait_patterns.compiled,
        &copy.enqueue_and_wait_patterns.compiled
    ));
    roles.merge(&copy).unwrap();
    let other = load_v2_config(&root, Some(&root.join("merge.yml"))).unwrap();
    roles.merge(&other.queues.factories).unwrap();
    assert!(roles.forbid_enqueue_and_wait_declarations);
    assert!(roles.is_wait_wrapper("enqueueOtherAndWait"));
    assert!(roles.is_wait_wrapper("submitReviewForResult"));
    assert!(!roles.is_wait_wrapper("enqueueReviewAndWaitForPurge"));
    assert_eq!(roles.enqueue_and_wait_patterns.patterns().len(), 2);
    let exact = load_v2_config(&root, Some(&root.join("exact.yml"))).unwrap();
    let encoded = serde_json::to_value(exact).unwrap();
    assert!(encoded["queues"]["factories"]
        .get("enqueueAndWaitPatterns")
        .is_none());
}

#[test]
fn malformed_pattern_is_a_configuration_error() {
    let root = root();
    let error = load_v2_config(&root, Some(&root.join("invalid-regex.yml"))).unwrap_err();
    assert!(
        format!("{error:#}").contains("enqueueAndWaitPatterns"),
        "{error:#}"
    );
}
