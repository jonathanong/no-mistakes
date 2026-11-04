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
        root.join(".no-mistakes/projects/one.yml"),
    ];
    let error = load_v2_config_with_path_from_visible(&root, None, &paths)
        .unwrap_err()
        .to_string();
    assert!(error.contains(".no-mistakes.yml") && error.contains("projects/one.yml"));
}

#[test]
fn typed_fragment_errors_point_to_authored_file_and_line() {
    let root = fixture("invalid-type");
    let path = root.join(".no-mistakes/tests/bad.yml");
    let error = load_v2_config_with_path_from_visible(&root, None, &[path])
        .unwrap_err()
        .to_string();
    assert!(error.contains(".no-mistakes/tests/bad.yml:4"), "{error}");
}

#[test]
fn an_empty_root_config_can_be_extended_by_fragments() {
    let root = fixture("empty-root");
    let visible = [".no-mistakes.yml", ".no-mistakes/rules/empty.yml"].map(|file| root.join(file));
    let (config, _) = load_v2_config_with_path_from_visible(&root, None, &visible).unwrap();
    assert!(config.rules.is_empty());
}

#[test]
fn declared_config_preserves_root_and_sorted_fragment_order() {
    let root = fixture("names");
    let config = super::declared_config(&root, None).unwrap();
    assert_eq!(config["effort"], "low");
    let names: Vec<_> = config["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|rule| rule["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["root", "alpha", "beta"]);
}

#[test]
fn named_duplicates_only_conflict_across_documents() {
    let root = fixture("names-duplicate");
    let a = root.join(".no-mistakes/rules/a.yml");
    let b = root.join(".no-mistakes/rules/b.yml");
    assert_eq!(
        load_v2_config_with_path_from_visible(&root, None, std::slice::from_ref(&a))
            .unwrap()
            .0
            .rules
            .len(),
        2
    );
    let error = load_v2_config_with_path_from_visible(&root, None, &[a, b])
        .unwrap_err()
        .to_string();
    assert!(error.contains("rules/a.yml") && error.contains("rules/b.yml"));
}

#[test]
fn nested_named_lists_belong_to_their_own_appended_rule() {
    let root = fixture("nested-names");
    let paths = [
        root.join(".no-mistakes/rules/a.yml"),
        root.join(".no-mistakes/rules/b.yml"),
    ];
    let (config, _) = load_v2_config_with_path_from_visible(&root, None, &paths).unwrap();
    assert_eq!(config.rules.len(), 2);
}

#[test]
fn scalar_section_folders_are_rejected_and_explicit_files_remain_whole() {
    let root = fixture("scalar-section");
    let error = load_v2_config_with_path_from_visible(
        &root,
        None,
        &[root.join(".no-mistakes/effort/value.yml")],
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("unknown config section `effort`"));
    let root = fixture("cases");
    let config =
        super::declared_config(&root, Some(&root.join(".no-mistakes/tests/custom.yml"))).unwrap();
    assert_eq!(config["rules"].as_array().unwrap().len(), 1);
}

#[test]
fn relative_path_order_is_lexical_across_nested_folders() {
    let config = super::declared_config(&fixture("path-order"), None).unwrap();
    let names: Vec<_> = config["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|rule| rule["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["flat", "nested"]);
}

#[test]
fn root_directory_documentation_is_not_a_config_fragment() {
    assert_eq!(
        super::declared_config(&fixture("readme"), None).unwrap(),
        serde_json::json!({})
    );
}

#[test]
fn hidden_organization_does_not_hide_section_files_or_unknown_sections() {
    let config = super::declared_config(&fixture("hidden-organization"), None).unwrap();
    assert_eq!(config["rules"][0]["name"], "hidden-group");
    let error = super::declared_config(&fixture("hidden-unknown"), None).unwrap_err();
    assert!(error.to_string().contains("unknown config section `rule`"));
}
