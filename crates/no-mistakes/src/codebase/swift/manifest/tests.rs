use super::*;

#[test]
fn manifest_targets_handle_nested_dependency_parentheses() {
    let source = r#"
        let package = Package(
            name: "Fixture",
            targets: [
                .target(
                    name: "VouchaFeatures",
                    dependencies: [
                        .product(name: "VouchaCore", package: "core"),
                        "VouchaAPI",
                    ]
                ),
                .testTarget(
                    name: "VouchaUITests",
                    dependencies: [
                        .target(name: "VouchaFeatures"),
                        .product(name: "VouchaModels", package: "core"),
                    ]
                ),
            ]
        )
    "#;

    let targets = parse_manifest_targets(source);
    let features = targets
        .iter()
        .find(|target| target.name == "VouchaFeatures")
        .expect("source target should parse");
    assert_eq!(
        features.dependencies,
        vec!["VouchaCore".to_string(), "VouchaAPI".to_string()]
    );

    let ui_tests = targets
        .iter()
        .find(|target| target.name == "VouchaUITests")
        .expect("test target should parse");
    assert!(ui_tests.is_test);
    assert_eq!(
        ui_tests.dependencies,
        vec!["VouchaFeatures".to_string(), "VouchaModels".to_string()]
    );
    assert_eq!(
        extract_test_target_names(source),
        vec!["VouchaUITests".to_string()]
    );
}

#[test]
fn manifest_targets_ignore_malformed_dependency_lists_and_calls() {
    let source = r#"
        let package = Package(
            name: "Fixture",
            targets: [
                .target(name: "NoDeps", dependencies: "not an array"),
                .testTarget(name: "Broken", dependencies: [.target(name: "NoDeps")
            ]
        )
    "#;

    let targets = parse_manifest_targets(source);
    let no_deps = targets
        .iter()
        .find(|target| target.name == "NoDeps")
        .expect("valid target should still parse");
    assert!(no_deps.dependencies.is_empty());
}
