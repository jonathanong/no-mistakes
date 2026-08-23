#[test]
fn eager_private_aggregate_resources_select_importing_test() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();
    for changed in [
        "resources/eager-object.txt",
        "resources/eager-static-field.txt",
    ] {
        let plan = generate_plan(&resource_plan_args(&root, root.join(changed))).unwrap();
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["eager-aggregate-consumer.test.ts"],
            "{changed} is read eagerly when the imported module initializes"
        );
        assert_eq!(plan.selected_tests[0].reasons[0].via, ["resource", "dependency"]);
    }
}

#[test]
fn uncalled_private_aggregate_method_resource_remains_pruned() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();
    let plan = generate_plan(&resource_plan_args(
        &root,
        root.join("resources/eager-deferred-method.txt"),
    ))
    .unwrap();

    assert!(plan.selected_tests.is_empty());
}
