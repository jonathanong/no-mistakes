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
fn exempts_remix_route_module_exports_only_under_configured_roots() {
    let findings = findings("unique-exports-remix");
    assert!(findings
        .iter()
        .any(|finding| finding.export_name == "SharedFlag"));
    assert!(findings
        .iter()
        .any(|finding| finding.export_name == "loader"
            && finding.file.starts_with("web/app/components/")));
    assert!(!findings.iter().any(|finding| {
        finding.file.contains("app/routes/")
            && matches!(finding.export_name.as_str(), "loader" | "action")
    }));
    assert!(remix::is_framework_export("loader", true));
    assert!(!remix::is_framework_export("loader", false));
    assert!(!remix::is_framework_export("SharedFlag", true));
}

#[test]
fn remix_package_dependency_does_not_enable_unique_export_exemptions() {
    // @remix-run/react in package.json is not a substitute for `type: remix`.
    let findings = findings("unique-exports-remix-unconfigured");
    assert!(findings
        .iter()
        .any(|finding| finding.export_name == "loader"));
}

#[test]
fn classifies_configured_remix_route_modules() {
    let root = PathBuf::from("/repo/web");
    let roots = [root.clone()];
    assert!(remix::is_route_module(&root.join("app/root.tsx"), &roots));
    assert!(remix::is_route_module(
        &root.join("app/routes/users.tsx"),
        &roots
    ));
    assert!(remix::is_route_module(
        &root.join("routes/legacy.tsx"),
        &roots
    ));
    assert!(!remix::is_route_module(
        &root.join("app/routes/users.server.ts"),
        &roots
    ));
    assert!(!remix::is_route_module(
        &root.join("app/routes/users.client.ts"),
        &roots
    ));
    assert!(!remix::is_route_module(
        &root.join("app/components/a.ts"),
        &roots
    ));
    assert!(!remix::is_route_module(
        &root.join("app/routes/readme.md"),
        &roots
    ));
    assert!(!remix::is_route_module(
        &root.join("app/routes/users.tsx"),
        &[]
    ));
}

#[test]
fn configured_remix_roots_ignore_non_remix_projects() {
    use crate::codebase::config::{Config, InferredRoots, ProjectConfig};
    use crate::config::v2::schema::ProjectType;
    use std::collections::HashMap;

    let workspace = Path::new("/repo");
    let empty = Config::default();
    assert!(remix::configured_roots(workspace, &empty, None).is_empty());

    let config = Config {
        projects: HashMap::from([
            (
                "web".to_string(),
                ProjectConfig {
                    type_: Some(ProjectType::Remix),
                    root: Some("web".to_string()),
                    ..Default::default()
                },
            ),
            (
                "lib".to_string(),
                ProjectConfig {
                    type_: Some(ProjectType::Library),
                    root: Some("lib".to_string()),
                    ..Default::default()
                },
            ),
            (
                "inferred".to_string(),
                ProjectConfig {
                    type_: Some(ProjectType::Remix),
                    ..Default::default()
                },
            ),
        ]),
        ..Default::default()
    };
    let inferred = InferredRoots {
        remix: Some(Some(workspace.join("app"))),
        ..Default::default()
    };
    let roots = remix::configured_roots(workspace, &config, Some(&inferred));
    assert!(roots.contains(&workspace.join("web")));
    assert!(roots.contains(&workspace.join("app")));
    let failed = InferredRoots {
        remix: Some(None),
        ..Default::default()
    };
    let fallback = Config {
        projects: HashMap::from([(
            "web".to_string(),
            ProjectConfig {
                type_: Some(ProjectType::Remix),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    assert_eq!(
        remix::configured_roots(workspace, &fallback, Some(&failed)),
        vec![workspace.to_path_buf()]
    );
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
