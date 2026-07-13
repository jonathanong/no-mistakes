use super::*;

fn load_config_projects(
    root: &Path,
    framework: Framework,
    raw: &str,
    path: &Path,
    source: &str,
    config_dir: &Path,
    tsconfig: &TsConfig,
) -> Result<Vec<ConfigProject>> {
    load_config_projects_inner(
        ConfigProjectInput {
            root,
            framework,
            raw,
            path,
            source,
            config_dir,
            tsconfig,
        },
        None,
    )
}

#[test]
fn glob_normalization_preserves_parent_segments_after_wildcards() {
    let wildcard_parent_glob = build_globset(&["*/../foo".to_string()]).unwrap();

    assert!(wildcard_parent_glob.is_match("pkg/../foo"));
    assert!(!wildcard_parent_glob.is_match("foo"));
}

#[test]
fn swift_load_projects_has_no_config_discovery_or_projects() {
    let root = Path::new("");
    let tsconfig = super::super::test_support::tsconfig_without_config(root);

    assert!(discovered_config_paths(root, Framework::Swift, &[]).is_empty());
    assert!(load_config_projects(
        root,
        Framework::Swift,
        "Package.swift",
        root,
        "",
        root,
        &tsconfig,
    )
    .unwrap()
    .is_empty());
}

#[test]
fn dotnet_load_projects_has_no_config_discovery_or_projects() {
    let root = Path::new("");
    let tsconfig = super::super::test_support::tsconfig_without_config(root);

    assert!(discovered_config_paths(root, Framework::Dotnet, &[]).is_empty());
    assert!(load_config_projects(
        root,
        Framework::Dotnet,
        "App.csproj",
        root,
        "",
        root,
        &tsconfig,
    )
    .unwrap()
    .is_empty());
}

#[test]
fn pass4b_runner_helpers_skip_ignored_candidates_for_visible_fallbacks() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4b-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(&root);
    let visible = visible_paths
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect();
    let tsconfig = super::super::test_support::tsconfig_without_config(&root);
    let playwright_path = root.join("runner/playwright.config.ts");
    let playwright_source = std::fs::read_to_string(&playwright_path).unwrap();
    let playwright = super::super::test_support::parse_playwright_from_visible(
        &playwright_source,
        &playwright_path,
        playwright_path.parent().unwrap(),
        &tsconfig,
        &visible,
    )
    .unwrap()
    .into_projects(&root, "runner/playwright.config.ts");
    let vitest_path = root.join("runner/vitest.config.ts");
    let vitest_source = std::fs::read_to_string(&vitest_path).unwrap();
    let vitest = super::super::test_support::parse_vitest_from_visible(
        &vitest_source,
        &vitest_path,
        vitest_path.parent().unwrap(),
        &root,
        &tsconfig,
        &visible,
    )
    .unwrap();

    assert_eq!(
        playwright[0].policy_name.as_deref(),
        Some("visible-playwright")
    );
    assert_eq!(vitest[0].policy_name.as_deref(), Some("visible-vitest"));
}
