use super::{load_v2_config_from_source_store, load_v2_config_with_path_from_visible};
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/config-fragments")
        .join(name)
}

#[test]
fn section_files_honor_visibility_and_source_store_memoization() {
    let root = fixture("valid");
    let visible =
        [".no-mistakes.yml", ".no-mistakes/rules/nested/extra.yml"].map(|name| root.join(name));
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&visible)));
    for _ in 0..2 {
        let config = load_v2_config_from_source_store(&root, None, &visible, &sources).unwrap();
        assert_eq!(config.rules.len(), 2);
    }
    assert_eq!(sources.physical_read_count(), 2);
    let config = load_v2_config_from_source_store(&root, None, &visible[..1], &sources).unwrap();
    assert_eq!(config.rules.len(), 1);
    assert_eq!(sources.physical_read_count(), 2);
}

#[test]
fn fragment_only_formats_and_ignored_files() {
    let root = fixture("cases");
    let visible = [
        "one.yaml",
        "two.json",
        "three.jsonc",
        "readme.md",
        ".ignored",
    ]
    .map(|name| root.join(".no-mistakes/rules").join(name));
    let (config, _) = load_v2_config_with_path_from_visible(&root, None, &visible).unwrap();
    assert!(config.rules.is_empty());
}

#[test]
fn explicit_fragment_path_ignores_all_other_files() {
    let root = fixture("valid");
    let explicit_root = fixture("cases");
    let path = explicit_root.join(".no-mistakes/tests/custom.yml");
    let (explicit, _) =
        load_v2_config_with_path_from_visible(&explicit_root, Some(&path), &[]).unwrap();
    assert_eq!(explicit.rules.len(), 1);
    let path = root.join(".no-mistakes/rules/nested/extra.yml");
    let root_config = root.join(".no-mistakes.yml");
    let (config, _) =
        load_v2_config_with_path_from_visible(&root, Some(&root_config), &[path]).unwrap();
    assert_eq!(config.rules.len(), 1);
}

#[test]
fn invalid_paths_and_duplicate_scalars_name_the_sources() {
    for (name, file, expected) in [
        ("direct", ".no-mistakes/config.yml", "directly inside"),
        (
            "bad-extension",
            ".no-mistakes/rules/invalid.txt",
            "unsupported config fragment",
        ),
    ] {
        let root = fixture(name);
        let error =
            load_v2_config_with_path_from_visible(&root, None, &[root.join(file)]).unwrap_err();
        assert!(error.to_string().contains(expected));
    }
    let root = fixture("scalar");
    let paths = [
        root.join(".no-mistakes.yml"),
        root.join(".no-mistakes/checks/one.yml"),
    ];
    let error = load_v2_config_with_path_from_visible(&root, None, &paths)
        .unwrap_err()
        .to_string();
    assert!(error.contains(".no-mistakes.yml") && error.contains("checks/one.yml"));
}
