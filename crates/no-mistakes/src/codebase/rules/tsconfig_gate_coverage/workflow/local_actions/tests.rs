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
        "name: Node\ndescription: Valid\nruns: {using: node24, main: dist/index.js}",
    ] {
        let tracked = if yaml.contains("using: node24") {
            &["action/dist/index.js"][..]
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
