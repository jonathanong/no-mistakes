use super::test_support::extract_test_target_names;
use super::*;

const EXTERNAL_BASE: &str = include_str!(
    "../../../../../../fixtures/test-plan/swift-manifest-diff/fixture/external-base.swift"
);
const EXTERNAL_CHANGED: &str = include_str!(
    "../../../../../../fixtures/test-plan/swift-manifest-diff/fixture/external-changed.swift"
);
const LOCAL_BASE: &str = include_str!(
    "../../../../../../fixtures/test-plan/swift-manifest-diff/fixture/local-base.swift"
);
const LOCAL_CHANGED: &str = include_str!(
    "../../../../../../fixtures/test-plan/swift-manifest-diff/fixture/local-changed.swift"
);
const PRODUCT_BASE: &str = include_str!(
    "../../../../../../fixtures/test-plan/swift-manifest-diff/fixture/product-base.swift"
);
const PRODUCT_CHANGED: &str = include_str!(
    "../../../../../../fixtures/test-plan/swift-manifest-diff/fixture/product-changed.swift"
);
const PLUGIN_BASE: &str = include_str!(
    "../../../../../../fixtures/test-plan/swift-manifest-diff/fixture/plugin-base.swift"
);
const PLUGIN_CHANGED: &str = include_str!(
    "../../../../../../fixtures/test-plan/swift-manifest-diff/fixture/plugin-changed.swift"
);
const MIXED_BASE: &str = include_str!(
    "../../../../../../fixtures/test-plan/swift-manifest-diff/fixture/mixed-base.swift"
);
const MIXED_CHANGED: &str = include_str!(
    "../../../../../../fixtures/test-plan/swift-manifest-diff/fixture/mixed-changed.swift"
);
const FORMATTING_BASE: &str = include_str!(
    "../../../../../../fixtures/test-plan/swift-manifest-diff/fixture/formatting-base.swift"
);
const FORMATTING_CHANGED: &str = include_str!(
    "../../../../../../fixtures/test-plan/swift-manifest-diff/fixture/formatting-changed.swift"
);
const DYNAMIC: &str =
    include_str!("../../../../../../fixtures/test-plan/swift-manifest-diff/fixture/dynamic.swift");

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
    assert_eq!(
        features.product_packages.get("VouchaCore"),
        Some(&"core".to_string())
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
fn manifest_products_map_product_names_to_module_targets() {
    let source = r#"
        let package = Package(products: [
            .library(name: "CoreProduct", targets: ["CoreModule", "Models"]),
            .executable(name: "Tool", targets: ["ToolMain"]),
        ])
    "#;

    assert_eq!(
        parse_manifest_products(source),
        std::collections::BTreeMap::from([
            (
                "CoreProduct".to_string(),
                vec!["CoreModule".to_string(), "Models".to_string()],
            ),
            ("Tool".to_string(), vec!["ToolMain".to_string()]),
        ])
    );
}

#[test]
fn local_package_and_product_identities_are_case_insensitive() {
    let source = r#"
        let package = Package(
            dependencies: [.package(path: "../Core")],
            targets: [
                .target(name: "App", dependencies: [
                    .product(name: "CoreProduct", package: "CORE"),
                ]),
            ]
        )
    "#;

    assert_eq!(
        parse_local_package_bindings(source).get("../Core"),
        Some(&"core".to_string())
    );
    assert_eq!(
        parse_manifest_targets(source)[0]
            .product_packages
            .get("CoreProduct"),
        Some(&"core".to_string())
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

#[test]
fn manifest_dependency_only_diff_accepts_static_dependency_surfaces() {
    for (before, after) in [
        (EXTERNAL_BASE, EXTERNAL_CHANGED),
        (LOCAL_BASE, LOCAL_CHANGED),
        (PRODUCT_BASE, PRODUCT_CHANGED),
        (PLUGIN_BASE, PLUGIN_CHANGED),
    ] {
        assert!(dependency_only_manifest_change(before, after).unwrap());
    }
    assert!(!dependency_only_manifest_change(FORMATTING_BASE, FORMATTING_CHANGED).unwrap());
}

#[test]
fn manifest_dependency_only_diff_keeps_mixed_configuration_broad() {
    assert!(!dependency_only_manifest_change(MIXED_BASE, MIXED_CHANGED).unwrap());
}

#[test]
fn manifest_normalization_preserves_whitespace_inside_static_strings() {
    let path_before = r#"let package = Package(dependencies: [.package(path: "../core lib")])"#;
    let path_after = r#"let package = Package(dependencies: [.package(path: "../core  lib")])"#;
    assert!(!formatting_only_manifest_change(path_before, path_after));
    assert!(dependency_only_manifest_change(path_before, path_after).unwrap());

    let name_before = r#"let package = Package(name: "Core Lib")"#;
    let name_after = r#"let package = Package(name: "Core  Lib")"#;
    assert!(!formatting_only_manifest_change(name_before, name_after));
    assert!(!dependency_only_manifest_change(name_before, name_after).unwrap());
}

#[test]
fn manifest_dependency_only_diff_diagnoses_dynamic_declarations() {
    assert_eq!(
        dependency_only_manifest_change(DYNAMIC, DYNAMIC).unwrap_err(),
        SwiftManifestDiagnostic::UnsupportedDynamicDeclaration
    );
}
