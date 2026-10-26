use super::*;
use crate::codebase::rules::source_store_for_files;
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    name: String,
    specifier: String,
    importer: String,
    expected: Option<String>,
    consumer: Option<String>,
    consumer_relative: Option<bool>,
}

fn fixture() -> PathBuf {
    crate::test_support::rule_fixture_root("unconstructed-error-class")
        .parent()
        .unwrap()
        .join("package-resolution/fixture")
}

fn saved_cases(name: &str) {
    let root = fixture();
    let expected_root = std::fs::canonicalize(&root).unwrap();
    let cases_store = source_store_for_files(&[]);
    let text = cases_store.read_path(&root.join(name)).unwrap();
    let cases: Vec<Case> = serde_json::from_str(&text).unwrap();
    for case in cases {
        let sources = source_store_for_files(&[]);
        let importer = root.join(&case.importer);
        let consumer = case.consumer.as_ref().map(|path| {
            if case.consumer_relative == Some(true) {
                PathBuf::from(path)
            } else {
                root.join(path)
            }
        });
        let context = consumer
            .as_deref()
            .or_else(|| importer.parent())
            .unwrap_or(&root);
        let actual = resolve_in_context(&case.specifier, &importer, context, &sources);
        let expected = case
            .expected
            .map(|path| crate::codebase::ts_resolver::normalize_path(&expected_root.join(path)));
        assert_eq!(actual, expected, "{}", case.name);
        let reads = sources.physical_read_count();
        let parses = sources.json_parse_count();
        assert_eq!(
            resolve_in_context(&case.specifier, &importer, context, &sources),
            actual,
            "{}",
            case.name
        );
        assert_eq!(
            sources.physical_read_count(),
            reads,
            "{} reread sources",
            case.name
        );
        assert_eq!(
            sources.json_parse_count(),
            parses,
            "{} reparsed manifests",
            case.name
        );
    }
}

#[test]
fn saved_declaration_resolution_cases_preserve_unknowns_and_source_ownership() {
    saved_cases("cases.json");
}

#[cfg(unix)]
#[test]
fn saved_symlink_contexts_never_select_a_different_benign_dependency() {
    saved_cases("symlink-cases.json");
}

#[test]
fn existing_relative_consumer_root_preserves_saved_declaration_lookup() {
    let root = fixture();
    let sources = source_store_for_files(&[]);
    let found = resolve_in_context("typed", &root.join("src/main.ts"), Path::new("."), &sources);
    let expected = std::fs::canonicalize(root)
        .unwrap()
        .join("node_modules/typed/types/base.d.ts");
    assert_eq!(found, Some(expected));
}

#[cfg(windows)]
#[test]
fn canonical_declaration_importer_can_chain_with_original_disk_consumer_root() {
    let canonical = std::fs::canonicalize(fixture()).unwrap();
    let text = canonical.to_str().unwrap();
    let root = if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{rest}"))
    } else {
        PathBuf::from(text.strip_prefix(r"\\?\").unwrap_or(text))
    };
    let sources = source_store_for_files(&[]);
    let first = resolve_in_context("typed", &root.join("src/main.ts"), &root, &sources).unwrap();
    let second = resolve_in_context("subpaths/part", &first, &root, &sources);
    assert_eq!(
        second,
        Some(canonical.join("node_modules/subpaths/part.d.ts"))
    );
}
