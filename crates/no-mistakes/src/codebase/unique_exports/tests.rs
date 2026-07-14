use super::*;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::workspaces::WorkspaceMap;

#[path = "tests/origin.rs"]
mod origin;
#[path = "tests/shared_facts_disable.rs"]
mod shared_facts_disable;
#[path = "tests/visibility.rs"]
mod visibility;

fn fixture(name: &str) -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

fn findings(name: &str) -> Vec<UniqueExportFinding> {
    analyze_project(&fixture(name), None, None).unwrap()
}

fn finding_names(findings: &[UniqueExportFinding]) -> Vec<(String, String)> {
    findings
        .iter()
        .map(|finding| (finding.export_name.clone(), finding.export_kind.clone()))
        .collect()
}

#[test]
fn pass4b_unique_origin_skips_ignored_local_and_workspace_candidates() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4b-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = normalize_path(fixture.path());
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(&root);
    let visible = visible_paths
        .iter()
        .map(|path| normalize_path(path))
        .collect::<HashSet<_>>();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig).with_visible(&visible);
    let workspace = crate::codebase::workspaces::load_from_files(&root, &visible_paths).unwrap();

    assert_eq!(
        super::origin::resolve_export_source(
            "./target",
            &root.join("unique/barrel.ts"),
            &resolver,
            &workspace,
        ),
        Some(root.join("unique/target.ts"))
    );
    assert_eq!(
        super::origin::resolve_export_source(
            "@fixture/pkg/feature",
            &root.join("impact/importer.ts"),
            &resolver,
            &workspace,
        ),
        Some(root.join("packages/pkg/src/feature.ts"))
    );
}

#[test]
fn reports_duplicate_value_and_type_exports_separately() {
    let findings = findings("unique-exports-basic");
    assert_eq!(findings.len(), 2);
    assert!(findings
        .iter()
        .any(|f| f.export_name == "shared" && f.export_kind == "value"));
    assert!(findings
        .iter()
        .any(|f| f.export_name == "SharedType" && f.export_kind == "type"));
    assert!(!findings.iter().any(|f| f.export_name == "default"));
}

#[test]
fn analyzes_project_from_shared_facts() {
    let root = fixture("unique-exports-basic");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            symbols: true,
            source: true,
            ..Default::default()
        },
    );

    let tsconfig = Path::new("tsconfig.json");
    let findings = analyze_project_with_facts(&root, None, Some(tsconfig), &facts).unwrap();

    assert_eq!(findings.len(), 2);
}

#[test]
fn prepared_entrypoints_match_shared_fact_analysis() {
    let root = fixture("unique-exports-basic");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            symbols: true,
            source: true,
            ..Default::default()
        },
    );
    let config = load_codebase_config_with_path(&root, None).unwrap();
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
        Some(Path::new("tsconfig.json")),
        &root,
        facts.files(),
    )
    .unwrap();
    let expected =
        analyze_project_with_facts(&root, None, Some(Path::new("tsconfig.json")), &facts).unwrap();

    let prepared = analyze_project_with_prepared_facts(&root, &config, &tsconfig, &facts).unwrap();

    assert_eq!(prepared, expected);

    // An unconfigured repository exercises the no-application prepared path.
    let config = crate::codebase::config::Config::default();
    let inferred = crate::codebase::config::InferredRoots::from_visible(&root, facts.files());
    let prepared = analyze_project_with_prepared_facts(&root, &config, &tsconfig, &facts).unwrap();
    let prepared_with_inferred = analyze_project_with_prepared_facts_and_inferred(
        &root, &config, &tsconfig, &facts, &inferred,
    )
    .unwrap();

    assert_eq!(prepared_with_inferred, prepared);
    assert_eq!(prepared_with_inferred.len(), 2);
}

#[test]
fn analyzes_nextjs_project_from_shared_facts() {
    let root = fixture("unique-exports-nextjs");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            symbols: true,
            source: true,
            ..Default::default()
        },
    );

    let findings = analyze_project_with_facts(&root, None, None, &facts).unwrap();

    assert!(findings
        .iter()
        .any(|finding| finding.export_name == "metadata"));
}

#[test]
fn analyze_project_with_facts_applies_options_per_rule_application() {
    let root = fixture("unique-exports-per-application-options");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            symbols: true,
            source: true,
            ..Default::default()
        },
    );

    let findings = analyze_project_with_facts(&root, None, None, &facts).unwrap();

    assert!(findings.iter().any(|finding| finding.file == "strict/a.ts"));
    assert!(!findings.iter().any(|finding| finding.file == "loose/a.ts"));
}

#[test]
fn top_level_rule_exclude_filters_unique_exports_input_files() {
    let findings = findings("unique-exports-path-filters");

    assert!(findings
        .iter()
        .any(|finding| finding.export_name == "shared"));
    assert!(!findings
        .iter()
        .any(|finding| finding.export_name == "Variants"));
}

#[test]
fn analyze_project_with_facts_applies_project_and_rule_path_filters() {
    let root = fixture("unique-exports-path-filters");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            symbols: true,
            source: true,
            ..Default::default()
        },
    );

    let findings = analyze_project_with_facts(&root, None, None, &facts).unwrap();

    assert!(findings
        .iter()
        .any(|finding| finding.export_name == "shared"));
    assert!(!findings
        .iter()
        .any(|finding| finding.export_name == "Variants"));
}

#[test]
fn analyze_project_with_facts_surfaces_per_application_errors() {
    let root = fixture("unique-exports-per-application-options");
    let facts = crate::codebase::check_facts::CheckFactMap {
        files: vec![root.join("strict/a.ts")],
        ..Default::default()
    };

    let error = analyze_project_with_facts(&root, None, None, &facts).unwrap_err();

    assert!(error.to_string().contains("missing shared facts"));
}

#[test]
fn analyze_project_with_facts_returns_empty_without_enabled_projects() {
    let root = fixture("unique-exports-config-disabled");
    let facts = crate::codebase::check_facts::CheckFactMap::default();

    let findings = analyze_project_with_facts(&root, None, None, &facts).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn analyze_project_with_facts_honors_disable_comments() {
    let root = fixture("unique-exports-disabled");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            symbols: true,
            source: true,
            ..Default::default()
        },
    );

    let findings = analyze_project_with_facts(&root, None, None, &facts).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn collect_source_files_from_facts_reports_missing_fact_shapes() {
    let root = fixture("unique-exports-basic");
    let file = root.join("src/a.ts");
    let files = vec![file.clone()];
    let missing = crate::codebase::check_facts::CheckFactMap::default();

    assert!(
        scan::collect_source_files_from_facts(&root, &files, &missing)
            .unwrap_err()
            .to_string()
            .contains("missing shared facts")
    );

    let mut parse_error = crate::codebase::check_facts::CheckFactMap::default();
    parse_error.ts.insert(
        file.clone(),
        crate::codebase::check_facts::CheckFileFacts {
            source: Some("export const Broken =".to_string()),
            parse_error: Some("bad syntax".to_string()),
            ..Default::default()
        },
    );
    assert!(
        scan::collect_source_files_from_facts(&root, &files, &parse_error)
            .unwrap_err()
            .to_string()
            .contains("bad syntax")
    );

    let mut missing_source = crate::codebase::check_facts::CheckFactMap::default();
    missing_source.ts.insert(file.clone(), Default::default());
    assert!(
        scan::collect_source_files_from_facts(&root, &files, &missing_source)
            .unwrap_err()
            .to_string()
            .contains("missing source facts")
    );

    let mut missing_symbols = crate::codebase::check_facts::CheckFactMap::default();
    missing_symbols.ts.insert(
        file,
        crate::codebase::check_facts::CheckFileFacts {
            source: Some("export const value = 1;".to_string()),
            ..Default::default()
        },
    );
    assert!(
        scan::collect_source_files_from_facts(&root, &files, &missing_symbols)
            .unwrap_err()
            .to_string()
            .contains("missing symbol facts")
    );
}

#[test]
fn root_is_normalized_before_analysis() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/unique-exports-basic/fixture/.");

    let findings = analyze_project(&root, None, None).unwrap();

    assert_eq!(findings.len(), 2);
    assert!(findings
        .iter()
        .all(|finding| !finding.file.starts_with('/')));
}

#[test]
fn strict_mode_reports_cross_type_duplicates() {
    let findings = findings("unique-exports-strict");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].export_name, "Shared");
    assert_eq!(findings[0].export_kind, "export");
    assert!(findings[0].message.starts_with("export `Shared`"));
    assert!(!findings[0].message.contains("export export"));
}

#[test]
fn follows_explicit_and_star_reexports() {
    let findings = findings("unique-exports-reexports");
    assert!(findings.is_empty());
}

#[test]
fn collapses_source_declarations_and_same_origin_barrels() {
    let findings = findings("unique-exports-barrels-pass");
    assert!(findings.is_empty());
}

#[test]
fn reports_distinct_declarations_even_when_reexported_through_barrels() {
    let findings = findings("unique-exports-real-duplicates");
    let names = finding_names(&findings);
    assert_eq!(findings.len(), 2);
    assert!(names.contains(&("Collision".to_string(), "value".to_string())));
    assert!(names.contains(&("Shape".to_string(), "type".to_string())));
}

#[test]
fn checks_only_projects_that_enable_the_rule() {
    let findings = findings("unique-exports-project-scope");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].export_name, "ScopedDuplicate");
    assert!(!findings
        .iter()
        .any(|finding| finding.export_name == "IgnoredDuplicate"));
}

#[test]
fn top_level_disabled_rule_overrides_project_scopes() {
    assert!(findings("unique-exports-project-scope-disabled").is_empty());
}

#[test]
fn keeps_type_and_value_exports_separate_by_default() {
    assert!(findings("unique-exports-type-value-split").is_empty());
}

#[test]
fn strict_mode_still_reports_cross_type_duplicates_after_origin_deduping() {
    let findings = findings("unique-exports-type-value-strict");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].export_name, "Shared");
    assert_eq!(findings[0].export_kind, "export");
}

#[test]
fn collapses_workspace_barrels_to_their_source_export() {
    assert!(findings("unique-exports-workspace-barrels").is_empty());
}

#[test]
fn project_scoping_preserves_workspace_resolution_outside_enabled_roots() {
    assert!(findings("unique-exports-project-scope-workspace").is_empty());
}

#[test]
fn honors_rule_disable_comments() {
    let findings = findings("unique-exports-disabled");
    assert!(findings.is_empty());
}

#[test]
fn exempts_known_nextjs_framework_exports_only_in_convention_files() {
    let findings = findings("unique-exports-nextjs");
    let metadata_count = findings
        .iter()
        .filter(|finding| finding.export_name == "metadata")
        .count();
    assert_eq!(metadata_count, 3);
    assert!(findings
        .iter()
        .any(|finding| finding.export_name == "metadata"
            && finding.file.starts_with("web/components/")));
    assert!(findings
        .iter()
        .any(|finding| finding.export_name == "metadata"
            && finding.file.starts_with("web/pages/app/")));
    assert!(
        findings
            .iter()
            .any(|finding| finding.export_name == "runtime"
                && finding.file.ends_with("page.test.tsx"))
    );
}

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

mod helper_edges;
