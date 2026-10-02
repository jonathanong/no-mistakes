use super::super::compile::compile;
use super::super::scan::scan;
use super::super::Options;
use crate::codebase::postgres::SchemaCatalog;
use std::path::PathBuf;

pub(super) const PATH: &str = "schemaCatalogPath: schema.json\n";

pub(super) fn messages(yaml: &str, body: serde_json::Value) -> Vec<String> {
    let options: Options = serde_yaml::from_str(yaml).unwrap();
    let compiled = compile(&options, None).unwrap();
    let mut root = body;
    if root.get("formatVersion").is_none() {
        root["formatVersion"] = serde_json::json!(2);
    }
    let catalog = SchemaCatalog::from_json(&root.to_string()).unwrap();
    scan(&catalog, &compiled, &options.schema_catalog_path)
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

pub(super) fn expect(yaml: &str, body: serde_json::Value, text: &str) {
    assert_eq!(messages(yaml, body), vec![format!("schema.json: {text}")]);
}

pub(super) fn expect_none(yaml: &str, body: serde_json::Value) {
    let messages = messages(yaml, body);
    assert!(messages.is_empty(), "{messages:#?}");
}

pub(super) fn expect_err(yaml: &str, snippet: &str) {
    let options: Options = serde_yaml::from_str(yaml).unwrap();
    let error = match compile(&options, None) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("expected config error containing {snippet}"),
    };
    assert!(error.contains(snippet), "{error}");
}

pub(super) fn catalog(name: &str) -> serde_json::Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/array-columns/catalogs")
        .join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}
