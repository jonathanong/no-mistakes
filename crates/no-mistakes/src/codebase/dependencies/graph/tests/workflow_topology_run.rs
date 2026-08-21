use super::*;

fn workflow_topology_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/workflow-topology"),
    )
}

#[test]
fn workflow_run_resolution_handles_cycles_cargo_and_unsafe_inputs() {
    let root = workflow_topology_fixture();
    let direct = root.join("scripts/direct.mjs");
    let cargo_file = root.join("src/bin/tool.rs");
    let universe = HashSet::from([
        root.join("package.json"),
        direct.clone(),
        cargo_file.clone(),
    ]);
    let mut bins = CargoBinIndex::default();
    bins.insert(None, "tool".to_string(), cargo_file.clone());
    let mut resolver = WorkflowRunResolver::new(&root, &universe, &bins, None);

    assert_eq!(
        resolver.resolve("npm run cycle-a", &root),
        vec![root.join("package.json")]
    );
    assert_eq!(
        resolver.resolve("cargo run --bin tool", &root),
        vec![cargo_file]
    );
    assert_eq!(
        resolver.resolve("node scripts/direct.mjs", &root),
        vec![direct.clone()]
    );
    assert!(resolver.resolve("node ${SCRIPT}", &root).is_empty());
    assert!(resolver
        .resolve("cargo run --bin tool | tee output; echo ok", &root)
        .is_empty());
    assert!(resolver
        .resolve("npm run missing", &root.parent().unwrap().join("outside"))
        .is_empty());
    assert!(resolver.resolve("ONLY_ENV=set", &root).is_empty());
    assert_eq!(resolver.nearest_package_json(root.parent().unwrap()), None);
    let mut targets = HashSet::new();
    resolver.resolve_cargo_targets(&[], &mut targets);
    resolver.resolve_package_script("${DYNAMIC}", &root, &mut HashSet::new(), &mut targets);
    resolver.insert_local_path(
        direct.to_str().expect("UTF-8 fixture path"),
        &root,
        &mut targets,
    );
    assert!(targets.contains(&direct));
}

#[test]
fn workflow_working_directory_and_local_action_resolution_are_scoped() {
    let root = workflow_topology_fixture();
    let workflow: serde_yaml::Value =
        serde_yaml::from_str("defaults:\n  run:\n    working-directory: scripts\n").unwrap();
    let job: serde_yaml::Value =
        serde_yaml::from_str("defaults:\n  run:\n    working-directory: packages/tool\n").unwrap();
    let step: serde_yaml::Value =
        serde_yaml::from_str("working-directory: .\nrun: node scripts/direct.mjs\n").unwrap();
    let empty: serde_yaml::Value = serde_yaml::from_str("{}").unwrap();

    assert_eq!(
        workflow_run_working_directory(&root, &workflow, &job, &step),
        Some(root.clone())
    );
    assert_eq!(
        workflow_run_working_directory(&root, &workflow, &job, &empty),
        Some(root.join("packages/tool"))
    );
    assert_eq!(
        workflow_run_working_directory(&root, &workflow, &empty, &empty),
        Some(root.join("scripts"))
    );
    let dynamic: serde_yaml::Value =
        serde_yaml::from_str("working-directory: ${{ matrix.dir }}").unwrap();
    assert_eq!(
        workflow_run_working_directory(&root, &empty, &empty, &dynamic),
        None
    );

    let action = root.join(".github/actions/local/action.yml");
    let universe = HashSet::from([action.clone()]);
    let action_dirs = [root.join(".github/actions")];
    assert_eq!(
        resolve_local_action_descriptor(&root, "./.github/actions/local", &universe, &action_dirs,),
        Some(action)
    );
    assert_eq!(
        resolve_local_action_descriptor(&root, "actions/checkout@v4", &universe, &action_dirs),
        None
    );
    assert_eq!(
        resolve_local_action_descriptor(&root, "./${{ matrix.action }}", &universe, &action_dirs,),
        None
    );
    assert_eq!(
        resolve_local_action_descriptor(&root, "./../outside", &universe, &action_dirs),
        None
    );
    let outside_action = root.join("other-actions/local/action.yml");
    assert_eq!(
        resolve_local_action_descriptor(
            &root,
            "./other-actions/local",
            &HashSet::from([outside_action]),
            &action_dirs,
        ),
        None
    );
}
