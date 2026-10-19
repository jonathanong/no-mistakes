use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::postgres::{
    EmbeddedSqlCall, EmbeddedSqlFileFacts, EmbeddedSqlKind, EmbeddedSqlOptions, EmbeddedSqlVariant,
    SchemaCatalog,
};
use crate::codebase::ts_source::{FileInventory, SourceStore};

const INSERT_SQL: &str = "INSERT INTO items (id) VALUES (1)";

fn compiled(unanalyzable_sql: &str) -> super::super::CompiledOptions {
    super::super::compile_options(&super::super::Options {
        schema_catalog_path: "schema.json".to_string(),
        unanalyzable_sql: unanalyzable_sql.to_string(),
        ..Default::default()
    })
    .unwrap()
}

fn call(sql_text: Option<&str>, variants: Vec<EmbeddedSqlVariant>) -> EmbeddedSqlCall {
    EmbeddedSqlCall {
        line: 1,
        sql_text: sql_text.map(str::to_string),
        kind: EmbeddedSqlKind::Inline,
        variants,
        ..Default::default()
    }
}

fn variant(sql: &str) -> EmbeddedSqlVariant {
    EmbeddedSqlVariant {
        sql_text: sql.to_string(),
        sql_source_offsets: Vec::new(),
        line: 1,
        sql_source_positions: Vec::new(),
        recovered_placeholder_positions: Vec::new(),
        append_sites: Vec::new(),
    }
}

fn facts(path: &Path, profile: &EmbeddedSqlOptions, calls: Vec<EmbeddedSqlCall>) -> CheckFactMap {
    let mut facts = CheckFactMap::default();
    facts.postgres_schema_catalogs.insert(
        "schema.json".to_string(),
        Ok(Arc::new(SchemaCatalog::default())),
    );
    let mut file = CheckFileFacts::default();
    file.embedded_sql.push((
        profile.clone(),
        EmbeddedSqlFileFacts {
            path: path.to_path_buf(),
            executor_bindings: Vec::new(),
            calls,
            call_spans: Vec::new(),
            exported_helpers: Default::default(),
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

fn scan(
    opts: &super::super::CompiledOptions,
    calls: Vec<EmbeddedSqlCall>,
) -> anyhow::Result<Vec<super::super::RuleFinding>> {
    let path = PathBuf::from("src/gap.ts");
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(std::slice::from_ref(
        &path,
    ))));
    let facts = facts(&path, &opts.embedded, calls);
    super::super::scan::scan_with_sources(
        Path::new("."),
        opts,
        std::slice::from_ref(&path),
        &sources,
        &facts,
    )
    .map(crate::codebase::rules::PostgresFindings::finish)
}

#[test]
fn opaque_inline_sql_continues_after_the_unanalyzable_finding() {
    let findings = scan(&compiled("fail"), vec![call(None, Vec::new())]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable-sql"));
}

#[test]
fn missing_statement_facts_for_an_insert_without_variants() {
    // A text-less call is a chunk boundary. The later insert still has to
    // fail closed when its prepared statement row was not indexed.
    let error = scan(
        &compiled("ignore"),
        vec![call(None, Vec::new()), call(Some(INSERT_SQL), Vec::new())],
    )
    .expect_err("missing facts");
    let message = error.to_string();
    assert!(
        message.contains("prepared SQL facts are missing"),
        "{message}"
    );
    assert!(!message.contains("variant"), "{message}");
}

#[test]
fn missing_statement_facts_for_an_insert_variant() {
    let error = scan(
        &compiled("ignore"),
        vec![call(Some(INSERT_SQL), vec![variant(INSERT_SQL)])],
    )
    .expect_err("missing variant facts");
    assert!(
        error
            .to_string()
            .contains("prepared SQL variant facts are missing"),
        "{error}"
    );
}
