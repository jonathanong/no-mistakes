use super::*;
use serde_yaml::Value;

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
        "name: Node 20\ndescription: Valid\nruns: {using: node20, main: dist/index.js}",
        "name: Node\ndescription: Valid\nruns: {using: node24, main: dist/index.js}",
    ] {
        let tracked = if yaml.contains("using: node") {
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
    for runtime in ["node12", "node16"] {
        let legacy = format!(
            "name: Legacy\ndescription: Invalid\nruns: {{using: {runtime}, main: dist/index.js}}"
        );
        assert!(!valid(
            &[("action", &legacy)],
            &["action/dist/index.js"],
            "action"
        ));
    }
    let local_node_hooks = "name: Node\ndescription: Invalid\nruns: {using: node24, pre: missing-pre.js, main: dist/index.js, post: cleanup.js}";
    assert!(!valid(
        &[("action", local_node_hooks)],
        &["action/dist/index.js"],
        "action"
    ));
    let local_node_post_hook = "name: Node\ndescription: Valid\nruns: {using: node24, main: dist/index.js, post: cleanup.js, post-if: always()}";
    assert!(valid(
        &[("action", local_node_post_hook)],
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
fn action_metadata_validates_all_supported_fields_before_cataloging() {
    let complete = "name: Complete\nauthor: Acme\ndescription: Valid\ninputs:\n  project:\n    description: Project name\n    required: true\n    default: app\n    deprecationMessage: Use matrix.project\noutputs:\n  result:\n    description: Result path\n    value: '${{ steps.result.outputs.path }}'\nruns:\n  using: composite\n  steps:\n    - id: result\n      run: echo ok\n      shell: bash\nbranding:\n  icon: check\n  color: green\n";
    assert!(valid(&[("action", complete)], &[], "action"));
    for icon in ["archive", "arrow-down-circle", "x", "zap-off", "zoom-out"] {
        let boundary_icon = format!(
            "name: Boundary\ndescription: Valid\nbranding: {{icon: {icon}, color: blue}}\nruns: {{using: composite, steps: [{{run: ok, shell: bash}}]}}"
        );
        assert!(
            valid(&[("action", &boundary_icon)], &[], "action"),
            "{icon}"
        );
    }
    for icon in ["not-a-feather-icon", "coffee"] {
        let invalid_icon = format!(
            "name: Invalid\ndescription: Invalid\nbranding: {{icon: {icon}, color: green}}\nruns: {{using: composite, steps: [{{run: ok, shell: bash}}]}}"
        );
        assert!(
            !valid(&[("action", &invalid_icon)], &[], "action"),
            "{icon}"
        );
    }
    let tolerated = "name: Tolerated\ndescription: Valid\nruns:\n  using: composite\n  steps:\n    - run: 'false'\n      shell: bash\n      continue-on-error: true\n";
    assert!(valid(&[("action", tolerated)], &[], "action"));

    for yaml in [
        "name: Unknown top-level\ndescription: Invalid\nunknown: true\nruns: {using: node24, main: index.js}",
        "name: Bad author\nauthor: [Acme]\ndescription: Invalid\nruns: {using: node24, main: index.js}",
        "name: Bad inputs\ndescription: Invalid\ninputs: []\nruns: {using: node24, main: index.js}",
        "name: Missing input description\ndescription: Invalid\ninputs: {project: {required: true}}\nruns: {using: node24, main: index.js}",
        "name: Bad input required\ndescription: Invalid\ninputs: {project: {description: Project, required: yes}}\nruns: {using: node24, main: index.js}",
        "name: Bad input default\ndescription: Invalid\ninputs: {project: {description: Project, default: [app]}}\nruns: {using: node24, main: index.js}",
        "name: Bad deprecation\ndescription: Invalid\ninputs: {project: {description: Project, deprecationMessage: [old]}}\nruns: {using: node24, main: index.js}",
        "name: Bad outputs\ndescription: Invalid\noutputs: []\nruns: {using: node24, main: index.js}",
        "name: Missing output description\ndescription: Invalid\noutputs: {result: {value: path}}\nruns: {using: composite, steps: [{run: ok, shell: bash}]}",
        "name: Missing composite output value\ndescription: Invalid\noutputs: {result: {description: Result}}\nruns: {using: composite, steps: [{run: ok, shell: bash}]}",
        "name: Bad output field\ndescription: Invalid\noutputs: {result: {description: Result, unknown: true, value: path}}\nruns: {using: composite, steps: [{run: ok, shell: bash}]}",
        "name: Bad branding\ndescription: Invalid\nbranding: {icon: check, color: pink}\nruns: {using: node24, main: index.js}",
        "name: Unknown branding icon\ndescription: Invalid\nbranding: {icon: not-a-feather-icon, color: green}\nruns: {using: node24, main: index.js}",
        "name: Omitted branding icon\ndescription: Invalid\nbranding: {icon: coffee, color: green}\nruns: {using: node24, main: index.js}",
        "name: Bad branding shape\ndescription: Invalid\nbranding: check\nruns: {using: node24, main: index.js}",
        "name: Unknown runs field\ndescription: Invalid\nruns: {using: node24, main: index.js, unknown: true}",
        "name: Unsupported local pre\ndescription: Invalid\nruns: {using: node24, pre: setup.js, main: index.js}",
        "name: Unsupported local pre condition\ndescription: Invalid\nruns: {using: node24, main: index.js, pre-if: always()}",
        "name: Bad node hook\ndescription: Invalid\nruns: {using: node24, main: index.js, post: [cleanup]}",
        "name: Bad docker args\ndescription: Invalid\nruns: {using: docker, image: alpine:3.22, args: [--ok, 1]}",
        "name: Bad docker env\ndescription: Invalid\nruns: {using: docker, image: alpine:3.22, env: [KEY]}",
        "name: Bad composite step\ndescription: Invalid\nruns: {using: composite, steps: [true]}",
    ] {
        assert!(!valid(&[("action", yaml)], &[], "action"), "{yaml}");
    }
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
