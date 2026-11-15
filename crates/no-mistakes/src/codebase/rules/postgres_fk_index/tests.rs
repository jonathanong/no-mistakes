use super::*;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{SqlForeignKeyMetadata, SqlSchemaFileFacts};
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::collections::BTreeMap;
use std::sync::Arc;

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

#[test]
fn missing_overlay_source_is_unanalyzable() {
    let root = std::env::temp_dir().join("nm-3387-fk-absent");
    let path = root.join("migrations").join("001.sql");
    let mut facts = CheckFactMap::default();
    facts.postgres.schema.insert(
        path.clone(),
        Ok(Arc::new(SqlSchemaFileFacts {
            path: path.clone(),
            ..Default::default()
        })),
    );
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[])));
    let findings = scan::scan(
        &root,
        &compile_options(&Options::default()),
        &[path],
        &sources,
        &facts,
    )
    .expect("prepared schema facts");
    assert!(
        findings.iter().any(|finding| {
            finding.target.as_deref() == Some("unanalyzable")
                && finding.message.contains("cannot read source")
        }),
        "{findings:?}"
    );
}
