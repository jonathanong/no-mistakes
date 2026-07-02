use super::*;
use no_mistakes::config::v2::{load_v2_config, NoMistakesConfig};
use std::path::{Path, PathBuf};

fn fixture(path: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases")
            .join(path)
            .join("fixture"),
    )
}

fn load_config(root: &Path) -> NoMistakesConfig {
    load_v2_config(root, None).unwrap()
}

#[test]
fn unique_exports_project_roots_cover_target_variants() {
    let root = fixture("check-discovery/unique-exports-target-roots");
    let config = load_config(&root);

    let roots = unique_exports_project_roots(&root, &config);

    assert_eq!(
        roots,
        vec![root.clone(), root.join("backend"), root.join("web")]
    );
}

#[test]
fn discover_check_files_includes_inferred_nextjs_project_files() {
    let root = fixture("config-v2/nextjs-inferred-root");
    let config = load_config(&root);

    let files = discover_check_files(&root, &config, &[], true);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_includes_inferred_remix_project_files() {
    let root = fixture("config-v2/remix-inferred-root");
    let config = load_config(&root);

    let files = discover_check_files(&root, &config, &[], true);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_includes_inferred_remix_vite_project_files() {
    let root = fixture("config-v2/remix-vite-inferred-root");
    let config = load_config(&root);

    let files = discover_check_files(&root, &config, &[], true);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_includes_inferred_vitejs_project_files() {
    let root = fixture("config-v2/vitejs-inferred-root");
    let config = load_config(&root);

    let files = discover_check_files(&root, &config, &[], true);

    assert!(files.iter().any(|path| path.ends_with("web/app/page.tsx")));
}

#[test]
fn discover_check_files_does_not_rescan_repository_root() {
    let root = fixture("check-discovery/repository-root-only");
    let config = load_config(&root);
    let mut expected = no_mistakes::codebase::ts_source::discover_files(&root, &[]);
    expected.sort();
    expected.dedup();

    let files = discover_check_files(&root, &config, &[], true);

    assert_eq!(files, expected);
}

#[test]
fn discover_check_files_preserves_included_fixture_roots() {
    let root = fixture("check-discovery/include-preserved-roots");
    let config = load_config(&root);

    let files = discover_check_files(&root, &config, &config.filesystem.skip_directories, false);

    assert!(files
        .iter()
        .any(|path| path.ends_with("fixtures/users.json")));
    assert!(files
        .iter()
        .any(|path| path.ends_with("web/fixtures/project-users.json")));
}

#[test]
fn literal_include_prefix_stops_before_brace_alternation() {
    assert_eq!(
        literal_include_prefix("docs/{a,b}/**"),
        Some(PathBuf::from("docs"))
    );
}

#[test]
fn include_preserved_roots_ignore_unknown_projects() {
    let root = PathBuf::from("/repo");
    let config = NoMistakesConfig {
        rules: vec![no_mistakes::config::v2::schema::RuleDef {
            rule: "test-email-domain-policy".to_string(),
            projects: vec!["missing".to_string()],
            include: vec!["fixtures/**".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };

    assert_eq!(
        include_preserved_roots(&root, &config),
        vec![root.join("fixtures")]
    );
}

#[test]
fn nextjs_project_without_single_config_root_is_ignored() {
    let root = fixture("check-discovery/nextjs-without-config");
    let config = load_config(&root);

    let roots = unique_exports_project_roots(&root, &config);

    assert!(roots.is_empty());
}
