use super::*;
use crate::codebase::postgres::SqlForeignKeyMetadata;
use std::collections::BTreeMap;

#[test]
fn empty_fk_columns_are_skipped() {
    let opts = compile_options(&Options::default());
    let fk = SqlForeignKeyMetadata {
        table_name: "comments".to_string(),
        column_names: Vec::new(),
        referenced_table_name: "posts".to_string(),
        delete_action: None,
        line: 1,
    };
    let findings = scan::scan_fk(
        "migrations/001.sql",
        "",
        &fk,
        &BTreeMap::new(),
        &opts,
        &mut Default::default(),
        &mut Default::default(),
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn directive_on_line_rejects_inputs_check_never_passes() {
    // compile_options replaces an empty allowDirective with fk-index-allow,
    // and extracted foreign keys never use line 0.
    assert!(!scan::directive_on_line("", 0, "fk-index-allow"));
    assert!(!scan::directive_on_line("post_id uuid", 1, ""));
}

#[test]
fn option_defaults_use_empty_includes() {
    let compiled = compile_options(&Options::default());
    assert!(compiled.allowed_columns.is_empty());
    assert!(compiled.allowed_tables.is_empty());
}
