use super::super::{compile_options, Options};
use std::path::Path;

#[test]
fn scanning_requires_prepared_facts() {
    let options: Options = serde_yaml::from_str("schemaCatalogPath: schema.json").unwrap();
    let compiled = compile_options(&options).unwrap();
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    let error = super::super::scan::scan(Path::new("."), &compiled, &[], &sources, None)
        .err()
        .unwrap();
    assert!(error
        .to_string()
        .contains("prepared PostgreSQL facts are required"));
}
