use super::*;

#[test]
fn action_directory_valid_reuses_the_root_cache() {
    let descriptors = descriptors(&[(
        "action",
        "name: Cached\ndescription: Valid\nruns: {using: composite, steps: [{run: ok, shell: bash}]}",
    )]);
    let tracked = BTreeSet::new();
    let mut cache = BTreeMap::new();
    assert!(action_directory_valid(
        "action",
        &descriptors,
        &tracked,
        &mut BTreeSet::new(),
        &mut cache,
    ));
    assert_eq!(cache.get("action"), Some(&true));
    assert!(action_directory_valid(
        "action",
        &descriptors,
        &tracked,
        &mut BTreeSet::new(),
        &mut cache,
    ));
}

#[test]
fn composite_action_self_cycle_is_rejected() {
    assert!(!valid(
        &[(
            "action",
            "name: Cyclic\ndescription: Valid\nruns: {using: composite, steps: [{uses: ./action}]}",
        )],
        &[],
        "action",
    ));
}

#[test]
fn docker_and_node_optional_hooks_must_exist_in_the_checkout() {
    assert!(valid(
        &[(
            "action",
            "name: Docker\ndescription: Valid\nruns: {using: docker, image: Dockerfile, pre-entrypoint: hook.sh}",
        )],
        &["action/Dockerfile", "action/hook.sh"],
        "action",
    ));
    assert!(!valid(
        &[(
            "action",
            "name: Docker\ndescription: Valid\nruns: {using: docker, image: Dockerfile, pre-entrypoint: missing.sh}",
        )],
        &["action/Dockerfile"],
        "action",
    ));
    assert!(valid(
        &[(
            "action",
            "name: Node\ndescription: Valid\nruns: {using: node20, main: dist/index.js, post: post.js}",
        )],
        &["action/dist/index.js", "action/post.js"],
        "action",
    ));
    assert!(!valid(
        &[(
            "action",
            "name: Node\ndescription: Valid\nruns: {using: node20, main: dist/index.js, post: missing.js}",
        )],
        &["action/dist/index.js"],
        "action",
    ));
}
