use super::*;

#[test]
fn checks_framework_named_exports_outside_nextjs_projects() {
    let findings = findings("unique-exports-not-next-app");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].export_name, "metadata");
    assert!(nextjs::is_framework_export(
        "web/app/page",
        "metadata",
        true
    ));
    assert!(!nextjs::is_framework_export(
        "web/pages/app/page.tsx",
        "metadata",
        true
    ));

    let next_root = fixture("unique-exports-nextjs");
    assert!(scan::package_json_has_next_dependency(
        &next_root.join("package.json")
    ));
    assert!(scan::test_support::file_is_in_nextjs_project(
        &next_root,
        &next_root.join("web/app/users/page.tsx")
    ));

    let not_next_root = fixture("unique-exports-not-next-app");
    assert!(!scan::test_support::file_is_in_nextjs_project(
        &not_next_root,
        Path::new("")
    ));
    assert!(!scan::package_json_has_next_dependency(
        &fixture("unique-exports-not-next-deps").join("package.json")
    ));
}

#[test]
fn checks_across_workspace_packages() {
    let findings = findings("unique-exports-workspace");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].export_name, "WorkspaceDuplicate");
}

#[test]
fn exempts_nextjs_metadata_asset_convention_exports() {
    let findings = findings("unique-exports-nextjs-assets");
    assert!(findings.iter().any(|finding| finding.export_name == "alt"));
    assert!(findings.iter().any(|finding| finding.export_name == "size"));
    assert!(findings
        .iter()
        .any(|finding| finding.export_name == "contentType"));
    assert!(!findings.iter().any(|finding| {
        finding.file.starts_with("web/app/")
            && matches!(
                finding.export_name.as_str(),
                "runtime" | "alt" | "size" | "contentType"
            )
    }));
}

#[test]
fn disabled_config_skips_rule() {
    assert!(findings("unique-exports-config-disabled").is_empty());
}

#[test]
fn explicit_tsconfig_resolves_path_aliases() {
    let root = fixture("unique-exports-tsconfig-paths");
    let findings = analyze_project(&root, None, Some(&root.join("tsconfig.json"))).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn relative_explicit_tsconfig_resolves_from_project_root() {
    let root = fixture("unique-exports-tsconfig-paths");
    let findings = analyze_project(&root, None, Some(Path::new("tsconfig.json"))).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn nearest_tsconfig_is_discovered_and_explicit_errors_are_reported() {
    let root = fixture("unique-exports-tsconfig-paths");
    let findings = analyze_project(&root, None, None).unwrap();
    assert!(findings.is_empty());
    assert!(analyze_project(&root, None, Some(&root.join("missing-tsconfig.json"))).is_err());
}

#[test]
fn covers_reexport_resolution_edge_cases() {
    let findings = findings("unique-exports-edge-cases");
    let names = finding_names(&findings);
    assert!(!names.contains(&("Direct".to_string(), "value".to_string())));
    assert!(!names.contains(&("DirectType".to_string(), "type".to_string())));
    assert!(!names.contains(&("DefaultAlias".to_string(), "value".to_string())));
    assert!(names.contains(&("DefaultShapeAlias".to_string(), "type".to_string())));
    assert!(!names.contains(&("ChainAlias".to_string(), "type".to_string())));
    assert!(!names.contains(&("StarResolved".to_string(), "value".to_string())));
    assert!(!names.contains(&("TypeStarOnly".to_string(), "type".to_string())));
    assert!(!names.contains(&("TypeStarValue".to_string(), "value".to_string())));
    assert!(names.contains(&("Namespace".to_string(), "value".to_string())));
    assert!(!names.contains(&("NamespacedOnly".to_string(), "value".to_string())));
    assert!(!names.contains(&("default".to_string(), "value".to_string())));
    assert!(names.contains(&("Hidden".to_string(), "value".to_string())));
    assert!(names.contains(&("Skipped".to_string(), "value".to_string())));
    assert!(names.contains(&("SameLine".to_string(), "value".to_string())));
}
