use super::super::compile::compile;
use super::super::scan::scan;
use super::super::Options;
use crate::codebase::postgres::SchemaCatalog;

pub(super) const PATH: &str = "schemaCatalogPath: schema.json\n";

pub(super) fn messages(yaml: &str, body: serde_json::Value) -> Vec<String> {
    let options: Options = serde_yaml::from_str(yaml).unwrap();
    let compiled = compile(&options).unwrap();
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
    let error = match compile(&options) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("expected config error containing {snippet}"),
    };
    assert!(error.contains(snippet), "{error}");
}

pub(super) fn column(table: &str, name: &str, data_type: &str) -> serde_json::Value {
    serde_json::json!({
        "tables": { table: { "columns": { name: { "dataType": data_type } } } }
    })
}
