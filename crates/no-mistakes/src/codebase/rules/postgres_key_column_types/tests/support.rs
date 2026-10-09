use super::super::{compile::compile, scan::scan};
use super::super::{Options, RULE_ID};
use crate::codebase::postgres::SchemaCatalog;
use std::path::PathBuf;

pub(super) const PATH: &str = "schemaCatalogPath: schema.json\n";

pub(super) fn findings(yaml: &str) -> Vec<crate::codebase::rules::RuleFinding> {
    let options: Options = serde_yaml::from_str(yaml).unwrap();
    let compiled = compile(&options, None).unwrap();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/key-column-types/catalogs/mixed.json");
    let source = std::fs::read_to_string(path).unwrap();
    let catalog = SchemaCatalog::from_json(&source).unwrap();
    scan(&catalog, &compiled, &options.schema_catalog_path).unwrap()
}

pub(super) fn expect_err(yaml: &str, snippet: &str) {
    let options: Options = serde_yaml::from_str(yaml).unwrap();
    let error = match compile(&options, None) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("expected config error containing {snippet}"),
    };
    assert!(error.contains(&format!("{RULE_ID} option")), "{error}");
    assert!(error.contains(snippet), "{error}");
}
