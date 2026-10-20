use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::postgres::{EmbeddedSqlCall, EmbeddedSqlFileFacts, EmbeddedSqlOptions};
use crate::codebase::ts_source::{FileInventory, SourceStore};

fn compiled() -> super::super::CompiledOptions {
    super::super::compile_options(&super::super::Options::default()).unwrap()
}

fn call(sql: &str) -> EmbeddedSqlCall {
    EmbeddedSqlCall {
        line: 4,
        sql_text: Some(sql.to_string()),
        ..Default::default()
    }
}

#[test]
fn prepared_metadata_falls_back_to_parsing_the_recovered_text() {
    let valid = super::findings_for_call(
        "lock.ts",
        "",
        &call("SELECT id FROM items WHERE id = 1 FOR UPDATE"),
        &compiled(),
        None,
    );
    assert!(
        valid
            .iter()
            .all(|finding| finding.target.as_deref() != Some(super::UNPARSEABLE_TARGET)),
        "{valid:#?}"
    );
    let invalid = super::findings_for_call(
        "lock.ts",
        "",
        &call("SELECT FROM items FOR UPDATE ("),
        &compiled(),
        None,
    );
    assert_eq!(invalid.len(), 1, "{invalid:#?}");
    assert_eq!(
        invalid[0].target.as_deref(),
        Some(super::UNPARSEABLE_TARGET)
    );
}

fn facts(path: &Path, profile: &EmbeddedSqlOptions, calls: Vec<EmbeddedSqlCall>) -> CheckFactMap {
    let mut facts = CheckFactMap::default();
    let mut file = CheckFileFacts::default();
    file.embedded_sql.push((
        profile.clone(),
        EmbeddedSqlFileFacts {
            path: path.to_path_buf(),
            executor_bindings: Vec::new(),
            calls,
            call_spans: Vec::new(),
            exported_helpers: Default::default(),
            exported_constants: Default::default(),
            exported_programs: Default::default(),
            pending_imported: Vec::new(),
            imported_call_lines: Default::default(),
            source_comment_spans: Vec::new(),
            fragments: Vec::new(),
            fragment_sites: Vec::new(),
            fragment_variants: Vec::new(),
            matched_factory_names: Vec::new(),
            matched_type_names: Vec::new(),
            pending_relative: Default::default(),
            unproven_member_query_spans: Vec::new(),
            postgres_rule_ids: Vec::new(),
            record_unproven_spans: true,
        },
    ));
    facts.ts.insert(path.to_path_buf(), Arc::new(file));
    facts.postgres.embedded.insert(
        (path.to_path_buf(), profile.clone()),
        Ok(Arc::new(Vec::new())),
    );
    facts
}

#[test]
fn a_recovered_statement_without_prepared_facts_is_an_error() {
    let opts = compiled();
    let path = PathBuf::from("src/lock.ts");
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(std::slice::from_ref(
        &path,
    ))));
    let facts = facts(&path, &opts.embedded, vec![call("SELECT 1")]);
    let error = super::super::scan::scan_with_sources(
        Path::new("."),
        &opts,
        std::slice::from_ref(&path),
        &sources,
        &facts,
    )
    .expect_err("missing prepared facts");
    assert!(
        error.to_string().contains("prepared SQL facts are missing"),
        "{error}"
    );
}
