use super::catalog_membership_work::WORK;
use super::*;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use crate::codebase::workspaces::IndexedWorkspaceMap;
use std::cell::Cell;

fn fixture(name: &str) -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig")
            .join(name),
    )
}

fn inputs(root: &Path) -> (Vec<PathBuf>, SourceStore) {
    let paths = crate::codebase::ts_source::discover_visible_paths(root);
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&paths)));
    (paths, sources)
}

fn compare(
    root: &Path,
    configs: &[PathBuf],
    paths: &[PathBuf],
    sources: &SourceStore,
) -> BTreeSet<PathBuf> {
    let workspace = IndexedWorkspaceMap::default();
    let expected =
        TsConfigCatalog::project_source_membership(root, configs, paths, sources, &workspace)
            .into_values()
            .flatten()
            .collect();
    let actual = TsConfigCatalog::project_source_union(root, configs, paths, sources, &workspace);
    assert_eq!(actual, expected);
    actual
}

#[test]
fn union_preserves_inherited_files_includes_and_requested_config_scope() {
    let root = fixture("membership-union");
    let (paths, sources) = inputs(&root);
    let configs = [
        "tsconfig.json",
        "tsconfig.overlap.json",
        "tsconfig.scripts.json",
    ]
    .map(|name| root.join(name));
    let members = compare(&root, &configs, &paths, &sources);
    let expected = [
        "src/covered.ts",
        "src/explicit.ts",
        "src/allowed.js",
        "scripts/tool.mts",
    ]
    .map(|name| root.join(name))
    .into_iter()
    .collect();
    // A referenced project is loaded, but cannot independently widen this union.
    assert_eq!(members, expected);
    let explicit_only = compare(
        &root,
        &[root.join("tsconfig.scripts.json")],
        &paths,
        &sources,
    );
    assert_eq!(
        explicit_only,
        BTreeSet::from([root.join("scripts/tool.mts")])
    );
    assert_eq!(compare(&root, &configs, &paths, &sources), members);
}

#[test]
fn union_matches_full_projection_for_invalid_and_empty_requests() {
    for name in [
        "invalid-json",
        "extends-cycle",
        "unresolved-package-extends",
        "directory-extends",
    ] {
        let root = fixture(name);
        let (paths, sources) = inputs(&root);
        compare(&root, &[root.join("tsconfig.json")], &paths, &sources);
        assert!(compare(&root, &[], &paths, &sources).is_empty());
    }
    let root = fixture("membership-union");
    let (_, sources) = inputs(&root);
    assert!(compare(&root, &[root.join("tsconfig.json")], &[], &sources).is_empty());
}

#[test]
fn coverage_consumer_normalizes_once_and_short_circuits_overlapping_configs() {
    let root = fixture("membership-union");
    let (paths, sources) = inputs(&root);
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    config.rules[0].include = vec!["src/**/*.ts".into()];
    WORK.with(|work| work.set((0, 0)));
    let findings = crate::codebase::rules::tsconfig_file_coverage::check_with_files_and_sources(
        &root, &config, &paths, &sources,
    )
    .unwrap();
    let (normalizations, checks) = WORK.with(Cell::get);
    // The previous map-then-flatten path normalizes and checks every config × file.
    assert_eq!(normalizations, paths.len());
    assert!(
        checks < 4 * paths.len(),
        "{checks} checks for {} files",
        paths.len()
    );
    let files = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect::<Vec<_>>();
    assert_eq!(files, ["src/excluded.ts", "src/generated/output.ts"]);
}
