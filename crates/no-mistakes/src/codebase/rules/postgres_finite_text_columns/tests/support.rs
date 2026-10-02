use super::super::compile::compile;
use super::super::scan::scan;
use super::super::Options;
use crate::codebase::postgres::SchemaCatalog;

pub(super) const PATH: &str = "schemaCatalogPath: schema.json\n";

pub(super) const NAMES: &str = "\
schemaCatalogPath: schema.json
namePatterns: ['(^|_)(status|state|kind|type|source|category|mode|channel)$']
";

pub(super) const REVIEW_OPTIONS: &str = "schemaCatalogPath: schema.json\ncolumnTypes: [text, character varying]\nnamePatterns: ['(^|_)(status|state|kind)$']\nskipGeneratedColumns: true\n";

pub(super) fn fixture(name: &str) -> serde_json::Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/finite-text")
        .join(name);
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

pub(super) fn messages(yaml: &str, body: serde_json::Value) -> Vec<String> {
    let options: Options = serde_yaml::from_str(yaml).unwrap();
    let compiled = compile(&options, None).unwrap();
    let catalog = SchemaCatalog::from_json(&body.to_string()).unwrap();
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
