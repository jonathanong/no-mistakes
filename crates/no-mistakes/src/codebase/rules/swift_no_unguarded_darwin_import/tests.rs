use super::conditions::parse_condition;
use super::imports::Proof;
use super::*;
use std::sync::Arc;

fn manifest_index(root: &Path, source: &str) -> packages::PackageIndex {
    packages::PackageIndex::from_manifests([(root.to_path_buf(), Some(Arc::from(source)))])
}

#[test]
fn swift_darwin_imports_report_unreadable_selected_sources() {
    let fixture = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/swift-no-unguarded-darwin-import"),
    );
    let root = fixture.join("fixture");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&fixture.join("configs/check.yml"))).unwrap();
    let files = [
        root.join("Package.swift"),
        root.join("Tests/ClockTests.swift"),
        root.join("Tests/ClockTests.swift/Unreadable.swift"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(findings[0].file, "Tests/ClockTests.swift");
    assert_eq!(findings[0].line, 2);
    assert_eq!(findings[1].file, "Tests/ClockTests.swift/Unreadable.swift");
    assert_eq!(findings[1].target.as_deref(), Some("unanalyzable"));
    assert!(findings[1].message.contains("cannot read source"));
}

#[test]
fn condition_parser_proves_negated_linux_and_leaves_other_negations_unproven() {
    assert_eq!(parse_condition(" !os(Linux)\n"), Proof::All);
    assert_eq!(parse_condition(" !( os(Linux) )\n"), Proof::None);
    assert_eq!(parse_condition(" !DEBUG\n"), Proof::None);
    assert_eq!(parse_condition(" !os(macOS)\n"), Proof::None);
}

#[test]
fn condition_parser_handles_apple_conjunctions_disjunctions_and_negations() {
    assert_eq!(
        parse_condition(" os(iOS) && !targetEnvironment(macCatalyst)\nimport UIKit"),
        Proof::All
    );
    assert_eq!(
        parse_condition(" !targetEnvironment(macCatalyst) && os(macOS)\n"),
        Proof::All
    );
    assert_eq!(
        parse_condition(" (os(iOS) || os(tvOS)) && !targetEnvironment(simulator)\n"),
        Proof::All
    );
    assert_eq!(
        parse_condition(" os(watchOS) && canImport(Security)\n"),
        Proof::All
    );
    assert_eq!(
        parse_condition(" os(visionOS)\n  && !targetEnvironment(simulator)\n"),
        Proof::All
    );
    assert_eq!(
        parse_condition(" canImport(Security) && canImport(UIKit)\n"),
        Proof::Modules(vec!["Security", "UIKit"])
    );
    assert_eq!(
        parse_condition(" (canImport(Security) && canImport(UIKit)) || canImport(Security)\n"),
        Proof::Modules(vec!["Security"])
    );
    assert_eq!(
        parse_condition(" os(macOS) || canImport(Darwin)\n"),
        Proof::Modules(vec!["Darwin"])
    );
    assert_eq!(
        parse_condition(" canImport(Darwin) || os(macOS)\n"),
        Proof::Modules(vec!["Darwin"])
    );
    assert_eq!(
        parse_condition(" canImport(Darwin) || canImport(Security)\n"),
        Proof::None
    );
    assert_eq!(
        parse_condition(" canImport(Darwin) || DEBUG\n"),
        Proof::None
    );
    assert_eq!(
        parse_condition(" !(os(iOS) && !targetEnvironment(macCatalyst))\n"),
        Proof::None
    );
    assert_eq!(parse_condition(" os(iOS) &&\n"), Proof::None);
    assert_eq!(parse_condition(" os(iOS) extra\n"), Proof::None);
    assert_eq!(parse_condition(" canImport(123)\n"), Proof::None);
    assert_eq!(
        parse_condition(" canImport(Security, _version: 1)\n"),
        Proof::Modules(vec!["Security"])
    );
    assert_eq!(parse_condition(" `DEBUG`\n"), Proof::None);
    assert_eq!(parse_condition(" (os(iOS)\n"), Proof::None);
    assert_eq!(parse_condition(" 123\n"), Proof::None);
    assert_eq!(parse_condition(" canImport(Darwin\n"), Proof::None);
    assert_eq!(parse_condition(" (os(iOS)\n  || os(macOS))\n"), Proof::All);
    assert_eq!(parse_condition(" os(iOS) ||\n  os(macOS)\n"), Proof::All);
    assert_eq!(parse_condition(" os(iOS) ||\n"), Proof::None);
    assert_eq!(parse_condition(" !\n"), Proof::None);
    assert_eq!(parse_condition(" (\n"), Proof::None);
    assert_eq!(parse_condition(" `DEBUG\n"), Proof::None);
    assert_eq!(
        parse_condition(" DEBUG && canImport(Security)\n"),
        Proof::Modules(vec!["Security"])
    );
    assert_eq!(
        parse_condition(" canImport(Security) && canImport(Security)\n"),
        Proof::Modules(vec!["Security"])
    );
}

#[test]
fn package_manifest_classifier_handles_platforms_targets_excludes_and_uncertainty() {
    let root = Path::new("/synthetic/pkg");
    let portable = manifest_index(
        root,
        r#"
        let package = Package(
            name: "Core",
            platforms: [],
            targets: [
                .target(name: "CoreLib"),
                .binaryTarget(name: "VendorBin", path: "Vendor.xcframework"),
                .testTarget(
                    name: "CoreTests",
                    path: "Tests/CoreTests",
                    exclude: ["Ignored.swift"],
                    sources: ["Included.swift", "Sub"]
                ),
            ]
        )
        "#,
    );
    assert!(matches!(
        portable.classify(&root.join("Tests/CoreTests/Included.swift")),
        FileScope::PortableTest
    ));
    assert!(matches!(
        portable.classify(&root.join("Tests/CoreTests/Sub/A.swift")),
        FileScope::PortableTest
    ));
    assert!(matches!(
        portable.classify(&root.join("Tests/CoreTests/Ignored.swift")),
        FileScope::Skip
    ));
    assert!(matches!(
        portable.classify(&root.join("Tests/CoreTests/Other.swift")),
        FileScope::Skip
    ));
    assert!(matches!(
        portable.classify(&root.join("Sources/CoreLib/File.swift")),
        FileScope::Skip
    ));
    assert!(matches!(
        portable.classify(&root.join("Package.swift")),
        FileScope::Skip
    ));

    let custom_linux = manifest_index(
        root,
        r#"
        let package = Package(
            name: "Mixed",
            platforms: [.macOS(.v14), .custom("linux", versionString: "1.0")],
            targets: [.testTarget(name: "MixedTests")]
        )
        "#,
    );
    assert!(matches!(
        custom_linux.classify(&root.join("Tests/MixedTests/A.swift")),
        FileScope::PortableTest
    ));

    let custom_apple = manifest_index(
        root,
        r#"
        let package = Package(
            name: "AppleCustom",
            platforms: [.custom("macOS", versionString: "14.0")],
            targets: [.testTarget(name: "AppleCustomTests")]
        )
        "#,
    );
    assert!(matches!(
        custom_apple.classify(&root.join("Tests/AppleCustomTests/A.swift")),
        FileScope::PortableTest
    ));

    let conditional = manifest_index(
        root,
        r#"
        #if os(macOS)
        let package = Package(name: "Cond", targets: [.testTarget(name: "CondTests")])
        #endif
        "#,
    );
    assert!(matches!(
        conditional.classify(&root.join("Tests/CondTests/A.swift")),
        FileScope::Uncertain(_)
    ));

    let mutated = manifest_index(
        root,
        r#"
        var package = Package(name: "Mut", targets: [.testTarget(name: "MutTests")])
        package.platforms = [.macOS(.v14)]
        "#,
    );
    assert!(matches!(
        mutated.classify(&root.join("Tests/MutTests/A.swift")),
        FileScope::Uncertain(_)
    ));

    let unparseable = manifest_index(root, "let x = 1");
    assert!(matches!(
        unparseable.classify(&root.join("Tests/Any/A.swift")),
        FileScope::Uncertain(_)
    ));

    let unreadable = packages::PackageIndex::from_manifests([(root.to_path_buf(), None)]);
    assert!(matches!(
        unreadable.classify(&root.join("Tests/Any/A.swift")),
        FileScope::Uncertain(_)
    ));
}

#[test]
fn package_manifest_classifier_handles_nested_packages_plugins_and_dynamic_fields() {
    let outer = Path::new("/synthetic/repo");
    let sibling = Path::new("/synthetic/other");
    let inner = Path::new("/synthetic/repo/SubPkg");
    let nested = packages::PackageIndex::from_manifests([
        (
            outer.to_path_buf(),
            Some(Arc::from(
                r#"
                import PackageDescription
                let helper = Helper.Package()
                let PackageInfo = 1
                let kind = Package.self
                let package = Package(
                    name: "Outer",
                    platforms,
                    myplatforms: [.iOS(.v17)],
                    platforms: [, .linux(.v1)],
                    targets: [
                        .plugin(name: "GenPlugin"),
                        .testTarget(name: "OuterTests")
                    ]
                )
                "#,
            )),
        ),
        (
            sibling.to_path_buf(),
            Some(Arc::from(r#"let package = Package(name: "Other")"#)),
        ),
        (
            inner.to_path_buf(),
            Some(Arc::from(
                r#"
                let package = Package(
                    name: "Inner",
                    platforms: [.iOS(.v17)],
                    targets: [.testTarget(name: "InnerTests")]
                )
                "#,
            )),
        ),
    ]);
    assert!(matches!(
        nested.classify(&outer.join("Plugins/GenPlugin/Plugin.swift")),
        FileScope::Skip
    ));
    assert!(matches!(
        nested.classify(&outer.join("Tests/OuterTests/A.swift")),
        FileScope::PortableTest
    ));
    assert!(matches!(
        nested.classify(&inner.join("Tests/InnerTests/A.swift")),
        FileScope::PortableTest
    ));

    let no_targets = manifest_index(outer, r#"let package = Package(name: "Empty")"#);
    assert!(matches!(
        no_targets.classify(&outer.join("Tests/Any/A.swift")),
        FileScope::Skip
    ));

    for bad_platform_manifest in [
        r#"let x = Package"#,
        r#"let package = Package(name: "P", platforms: /* empty */, targets: [.testTarget(name: "T")])"#,
        r#"let package = Package(name: "P", platforms: [dynPlatform], targets: [.testTarget(name: "T")])"#,
        r#"let package = Package(name: "P", platforms: [.iOS(.v17) + extra], targets: [.testTarget(name: "T")])"#,
        r#"let package = Package(name: "P", platforms: [.iOS(.v17], targets: [.testTarget(name: "T")])"#,
        r#"let package = Package(name: "P", platforms: [.unknownOs(.v1)], targets: [.testTarget(name: "T")])"#,
        r#"let package = Package(name: "P", platforms: [.custom(dynName, versionString: "1")], targets: [.testTarget(name: "T")])"#,
        r#"let package = Package(name: "P", platforms: [.custom("solaris", versionString: "1")], targets: [.testTarget(name: "T")])"#,
    ] {
        let index = manifest_index(outer, bad_platform_manifest);
        assert!(matches!(
            index.classify(&outer.join("Tests/T/A.swift")),
            FileScope::Uncertain(_)
        ));
    }

    for bad_target_manifest in [
        r#"let package = Package(name: "P", targets: [dynamicTarget])"#,
        r#"let package = Package(name: "P", targets: [.futureTarget(name: "X")])"#,
        r#"let package = Package(name: "P", targets: [.testTarget(name: /* empty */)])"#,
        r#"let package = Package(name: "P", targets: [.testTarget(path: "Tests/X")])"#,
        r#"let package = Package(name: "P", targets: [.testTarget(name: "X", path: dynPath)])"#,
        r#"let package = Package(name: "P", targets: [.testTarget(name: "X", exclude: dynEx)])"#,
        r#"let package = Package(name: "P", targets: [.testTarget(name: "X", sources: dynSrc)])"#,
    ] {
        let index = manifest_index(outer, bad_target_manifest);
        assert!(matches!(
            index.classify(&outer.join("Tests/X/A.swift")),
            FileScope::Uncertain(_)
        ));
    }
}
