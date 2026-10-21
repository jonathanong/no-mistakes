use super::*;
use crate::codebase::postgres::EmbeddedSqlCall;
use std::path::PathBuf;

#[test]
fn source_scan_classifies_env_named_and_skips_validators() {
    let mut out = BTreeSet::new();
    scan_source(
        "query(`SELECT id FROM ${process.env.SCHEMA}.users WHERE id = ${id}`)",
        &mut out,
    );
    assert!(out.contains(&Provenance::ProcessEnv("SCHEMA".into())));
    assert!(out.contains(&Provenance::Named("id".into())));
    let mut validated = BTreeSet::new();
    scan_source(
        "query(`SELECT id FROM ${schemaOf(process.env.SCHEMA ?? \"\")}.users ${unclosed`)",
        &mut validated,
    );
    assert!(validated.is_empty());
    let source = "query(`SELECT id FROM ${tables.users}`)";
    let owned = EmbeddedSqlCall::default();
    let found = interpolations(&SqlCall {
        file: PathBuf::from("query.ts"),
        span: (0, source.len() as u32),
        call: &owned,
        source: Some(source),
    });
    assert!(found.contains(&Provenance::Named("tables".into())));
    assert!(interpolations(&SqlCall {
        file: PathBuf::from("query.ts"),
        span: (0, 4),
        call: &owned,
        source: None,
    })
    .is_empty());
}
