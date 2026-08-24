use super::*;

#[test]
fn collect_swift_facts_returns_empty_without_configured_packages() {
    let root = Path::new("/repo");
    assert!(collect_swift_facts(root, &[], &[]).files.is_empty());
}

#[test]
fn collect_swift_facts_returns_empty_when_packages_do_not_parse() {
    let root = Path::new("/repo");
    let files = vec![PathBuf::from("/repo/Client/Sources/App/App.swift")];
    assert!(collect_swift_facts(root, &files, &["Client".to_string()])
        .files
        .is_empty());
}

#[test]
fn target_index_prefers_the_deepest_matching_target_root() {
    let file = PathBuf::from("/repo/Client/Sources/App/Generated/Client.swift");
    let package = SwiftPackageFacts {
        package_root: PathBuf::from("/repo/Client"),
        local_package_paths: Vec::new(),
        local_package_bindings: BTreeMap::new(),
        products: BTreeMap::new(),
        targets: BTreeMap::from([
            (
                "App".to_string(),
                SwiftTargetFacts {
                    name: "App".to_string(),
                    roots: vec![PathBuf::from("/repo/Client/Sources/App")],
                    ..Default::default()
                },
            ),
            (
                "Generated".to_string(),
                SwiftTargetFacts {
                    name: "Generated".to_string(),
                    roots: vec![PathBuf::from("/repo/Client/Sources/App/Generated")],
                    ..Default::default()
                },
            ),
        ]),
    };

    let index = target_index(&[package], std::slice::from_ref(&file));

    assert_eq!(index.get(&file).map(String::as_str), Some("Generated"));
}

#[test]
fn package_manifests_are_not_indexed_as_swift_source_symbols() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/swift-native-topology/fixture");
    let manifest = root.join("swift-clients/core/Package.swift");
    let source = root.join("swift-clients/core/Sources/VouchaCore/APIClient.swift");
    let facts = collect_swift_facts(
        &root,
        &[manifest.clone(), source.clone()],
        &["swift-clients/core".to_string()],
    );
    assert!(!facts.files.contains_key(&manifest));
    assert!(facts.files.contains_key(&source));
}

#[test]
fn executable_and_custom_targets_own_their_configured_source_roots() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/swift-target-ownership/fixture");
    let core = root.join("Sources/Core/Core.swift");
    let runner = root.join("Tools/Runner/Runner.swift");
    let custom_test = root.join("Checks/Integration/CustomTests.swift");
    let plugin = root.join("Tooling/Plugin/Plugin.swift");
    let facts = collect_swift_facts(
        &root,
        &[
            core.clone(),
            runner.clone(),
            custom_test.clone(),
            plugin.clone(),
        ],
        &[".".to_string()],
    );

    assert_eq!(facts.files[&core].target.as_deref(), Some("Core"));
    assert_eq!(facts.files[&runner].target.as_deref(), Some("Runner"));
    assert_eq!(
        facts.files[&custom_test].target.as_deref(),
        Some("CustomTests")
    );
    assert_eq!(facts.files[&plugin].target.as_deref(), Some("Plugin"));
}
