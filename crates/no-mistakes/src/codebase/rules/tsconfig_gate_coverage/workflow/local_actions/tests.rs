use super::*;

fn descriptors(entries: &[(&str, &str)]) -> BTreeMap<String, Value> {
    entries
        .iter()
        .map(|(directory, yaml)| {
            (
                (*directory).to_string(),
                serde_yaml::from_str(yaml).expect("valid fixture YAML"),
            )
        })
        .collect()
}

fn valid(entries: &[(&str, &str)], tracked: &[&str], directory: &str) -> bool {
    action_directory_valid(
        directory,
        &descriptors(entries),
        &tracked.iter().map(|path| (*path).to_string()).collect(),
        &mut BTreeSet::new(),
        &mut BTreeMap::new(),
    )
}

#[test]
fn action_metadata_requires_a_supported_complete_execution_contract() {
    for yaml in [
        "name: Composite\ndescription: Valid\nruns: {using: composite, steps: [{run: ok, shell: bash}]}",
        "name: Docker\ndescription: Valid\nruns: {using: docker, image: Dockerfile}",
        "name: Container\ndescription: Valid\nruns: {using: docker, image: 'docker://alpine:3.22'}",
        "name: Upper container\ndescription: Valid\nruns: {using: docker, image: 'DOCKER://alpine:3.22'}",
        "name: Base image\ndescription: Valid\nruns: {using: docker, image: 'alpine:3.22'}",
        "name: Node\ndescription: Valid\nruns: {using: node24, main: dist/index.js}",
    ] {
        let tracked = if yaml.contains("using: node24") {
            &["action/dist/index.js"][..]
        } else if yaml.contains("image: Dockerfile") {
            &["action/Dockerfile"][..]
        } else {
            &[]
        };
        assert!(valid(&[("action", yaml)], tracked, "action"), "{yaml}");
    }

    for yaml in [
        "[]",
        "description: Missing name\nruns: {using: node20, main: index.js}",
        "name: Blank description\ndescription: '  '\nruns: {using: node20, main: index.js}",
        "name: Missing runs\ndescription: Invalid",
        "name: Empty composite\ndescription: Invalid\nruns: {using: composite, steps: []}",
        "name: Empty step\ndescription: Invalid\nruns: {using: composite, steps: [{}]}",
        "name: Missing shell\ndescription: Invalid\nruns: {using: composite, steps: [{run: ok}]}",
        "name: Unsupported step field\ndescription: Invalid\nruns: {using: composite, steps: [{run: ok, shell: bash, timeout-minutes: 1}]}",
        "name: Empty image\ndescription: Invalid\nruns: {using: docker, image: ''}",
        "name: Padded Dockerfile\ndescription: Invalid\nruns: {using: docker, image: ' Dockerfile '}",
        "name: Empty container\ndescription: Invalid\nruns: {using: docker, image: 'docker://'}",
        "name: Missing Dockerfile\ndescription: Invalid\nruns: {using: docker, image: Dockerfile}",
        "name: Empty main\ndescription: Invalid\nruns: {using: node20, main: ''}",
        "name: Future runtime\ndescription: Invalid\nruns: {using: node99, main: index.js}",
    ] {
        assert!(!valid(&[("action", yaml)], &[], "action"), "{yaml}");
    }

    let node = "name: Node\ndescription: Valid\nruns: {using: node24, main: dist/index.js}";
    assert!(!valid(&[("action", node)], &[], "action"));
    assert!(!valid(
        &[("action", node)],
        &["outside/dist/index.js"],
        "action"
    ));
    let escaping = "name: Node\ndescription: Invalid\nruns: {using: node24, main: ../outside.js}";
    assert!(!valid(&[("action", escaping)], &["outside.js"], "action"));
    let absolute = "name: Node\ndescription: Invalid\nruns: {using: node24, main: /outside.js}";
    assert!(!valid(
        &[("action", absolute)],
        &["action/outside.js"],
        "action"
    ));
    let local_node_hooks = "name: Node\ndescription: Valid\nruns: {using: node24, pre: missing-pre.js, main: dist/index.js, post: missing-post.js}";
    assert!(valid(
        &[("action", local_node_hooks)],
        &["action/dist/index.js"],
        "action"
    ));
    let escaping_dockerfile =
        "name: Docker\ndescription: Invalid\nruns: {using: docker, image: ../Dockerfile}";
    assert!(!valid(
        &[("action", escaping_dockerfile)],
        &["Dockerfile"],
        "action"
    ));
    let suffixed_dockerfile =
        "name: Docker\ndescription: Valid\nruns: {using: docker, image: build/Dockerfile.test}";
    assert!(valid(
        &[("action", suffixed_dockerfile)],
        &["action/build/Dockerfile.test"],
        "action"
    ));
}

#[test]
fn composite_actions_require_valid_acyclic_nested_local_targets() {
    let root = "name: Root\ndescription: Root\nruns: {using: composite, steps: [{uses: ./nested}]}";
    let nested = "name: Nested\ndescription: Nested\nruns: {using: composite, steps: [{run: ok, shell: bash}]}";
    assert!(valid(&[("root", root), ("nested", nested)], &[], "root"));
    assert!(!valid(&[("root", root)], &[], "root"));

    let cycle =
        "name: Nested\ndescription: Nested\nruns: {using: composite, steps: [{uses: ./root}]}";
    assert!(!valid(&[("root", root), ("nested", cycle)], &[], "root"));
}

#[test]
fn composite_actions_reject_unconditional_static_failures() {
    for (condition, run) in [
        ("", "false"),
        ("", "exit 1"),
        ("", "return"),
        ("if: true\n      ", "false"),
        ("if: '${{ vars.MAYBE }}'\n      ", "false"),
    ] {
        let action = format!(
            "name: Failing\ndescription: Failing\nruns:\n  using: composite\n  steps:\n    - {condition}shell: bash\n      run: '{run}'\n"
        );
        assert!(!valid(&[("action", &action)], &[], "action"), "{run}");
    }
    for step in [
        "{run: echo ok, shell: bash}",
        "{run: 'exit 0', shell: bash}",
        "{if: false, run: 'false', shell: bash}",
    ] {
        let action = format!(
            "name: Passing\ndescription: Passing\nruns: {{using: composite, steps: [{step}]}}"
        );
        assert!(valid(&[("action", &action)], &[], "action"), "{step}");
    }
}

#[test]
fn composite_action_nesting_is_bounded() {
    let descriptors = |count: usize| {
        (0..count)
            .map(|index| {
                let runs = if index + 1 == count {
                    "runs: {using: composite, steps: [{run: echo ok, shell: bash}]}".to_string()
                } else {
                    format!(
                        "runs: {{using: composite, steps: [{{uses: ./action-{}}}]}}",
                        index + 1
                    )
                };
                (
                    format!("action-{index}"),
                    serde_yaml::from_str(&format!(
                        "name: Action {index}\ndescription: Nested\n{runs}"
                    ))
                    .unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>()
    };

    assert!(action_directory_valid(
        "action-0",
        &descriptors(10),
        &BTreeSet::new(),
        &mut BTreeSet::new(),
        &mut BTreeMap::new(),
    ));
    assert!(!action_directory_valid(
        "action-0",
        &descriptors(11),
        &BTreeSet::new(),
        &mut BTreeSet::new(),
        &mut BTreeMap::new(),
    ));
}
