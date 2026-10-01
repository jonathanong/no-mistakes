use super::*;
use anyhow::Context;

impl SchemaCatalog {
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let snapshot: super::snapshot::Snapshot =
            serde_json::from_str(json).context("schema catalog JSON is not valid")?;
        if snapshot.format_version != 2 {
            anyhow::bail!("schema catalog JSON must use formatVersion 2");
        }
        Self::from_snapshot("schema.json", snapshot)
    }
}

mod edges;
mod errors;
mod expressions;
mod findings;
mod function_body;
mod model;
mod order;
mod parse;
mod relation;
mod resolve;

fn load_fixture(name: &str) -> Result<SchemaCatalog, String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres/catalog");
    let path = root.join(name);
    let sources = SourceStore::new(std::sync::Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&[path]),
    ));
    SchemaCatalog::load(&root, name, &sources).map_err(|error| error.to_string())
}

#[test]
fn expressions_ignore_formatting_but_not_source_bindings() {
    assert!(expression_matches(
        "LOWER( category_text )",
        "lower(category_text)",
        false
    ));
    assert!(!expression_matches(
        "lower(input.category_text)",
        "lower(category_text)",
        false
    ));
    assert!(expression_matches(
        "lower(input.category_text)",
        "lower(category_text)",
        true
    ));
}
#[test]
fn expression_normalization_preserves_structural_parentheses() {
    assert_ne!(
        normalize_expression("(left_value + right_value) * multiplier"),
        normalize_expression("left_value + (right_value * multiplier)")
    );
}
#[test]
fn rejects_outside_catalog_paths() {
    let root = Path::new("/repo");
    assert!(catalog_path(root, "../schema.json").is_err());
    assert!(catalog_path(root, "/schema.json").is_err());
    assert_eq!(
        catalog_path(root, "db/schema.json").unwrap(),
        root.join("db/schema.json")
    );
    assert_eq!(
        catalog_path(root, "./db/schema.json").unwrap(),
        root.join("db/schema.json")
    );
}

#[test]
fn reports_catalog_read_parse_and_version_failures() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres/catalog");
    let paths = [root.join("invalid.json"), root.join("wrong-version.json")];
    let sources = SourceStore::new(std::sync::Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&paths),
    ));

    let malformed = SchemaCatalog::load(&root, "invalid.json", &sources).unwrap_err();
    assert!(malformed.to_string().contains("not valid JSON"));
    let wrong_version = SchemaCatalog::load(&root, "wrong-version.json", &sources).unwrap_err();
    assert!(wrong_version.to_string().contains("formatVersion 2"));
    let missing = SchemaCatalog::load(&root, "missing.json", &sources).unwrap_err();
    assert!(missing
        .to_string()
        .contains("failed to read schemaCatalogPath"));
}
