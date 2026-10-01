use super::super::compile::compile;
use super::super::scan::scan;
use super::super::Options;
use crate::codebase::postgres::SchemaCatalog;

pub(super) const PATH: &str = "schemaCatalogPath: schema.json\n";

pub(super) const NAMES: &str = "\
schemaCatalogPath: schema.json
namePatterns: ['(^|_)(status|state|kind|type|source|category|mode|channel)$']
";

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

pub(super) fn checked(
    table: &str,
    column: &str,
    data_type: &str,
    definition: &str,
) -> serde_json::Value {
    serde_json::json!({
        "tables": {
            table: {
                "columns": { column: { "dataType": data_type } },
                "checkConstraints": { "ck": { "definition": definition } }
            }
        }
    })
}
