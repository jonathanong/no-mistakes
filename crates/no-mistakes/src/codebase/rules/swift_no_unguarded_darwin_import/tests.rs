use super::conditions::parse_condition;
use super::imports::Proof;
use super::*;

#[test]
fn swift_darwin_imports_report_unreadable_selected_sources() {
    let fixture = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/swift-no-unguarded-darwin-import"),
    );
    let root = fixture.join("fixture");
    let config = crate::config::v2::load_v2_config(&root, Some(&fixture.join("configs/check.yml")))
        .unwrap();
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
    assert_eq!(parse_condition(" canImport(Darwin) || DEBUG\n"), Proof::None);
    assert_eq!(
        parse_condition(" !(os(iOS) && !targetEnvironment(macCatalyst))\n"),
        Proof::None
    );
    assert_eq!(parse_condition(" os(iOS) &&\n"), Proof::None);
    assert_eq!(parse_condition(" os(iOS) extra\n"), Proof::None);
    assert_eq!(parse_condition(" canImport(123)\n"), Proof::None);
    assert_eq!(parse_condition(" canImport(Security, _version: 1)\n"), Proof::Modules(vec!["Security"]));
    assert_eq!(parse_condition(" `DEBUG`\n"), Proof::None);
    assert_eq!(parse_condition(" (os(iOS)\n"), Proof::None);
}

#[test]
fn package_manifest_classifier_handles_platforms_targets_excludes_and_uncertainty() {
    let root = Path::new("/synthetic/pkg");
    let portable = packages::parse_manifest_for_test(
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

    let custom_linux = packages::parse_manifest_for_test(
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

    let custom_apple = packages::parse_manifest_for_test(
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
        FileScope::Skip
    ));

    let conditional = packages::parse_manifest_for_test(
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

    let mutated = packages::parse_manifest_for_test(
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

    let unparseable = packages::parse_manifest_for_test(root, "let x = 1");
    assert!(matches!(
        unparseable.classify(&root.join("Tests/Any/A.swift")),
        FileScope::Uncertain(_)
    ));
}
