use super::*;
use crate::codebase::postgres::{extract_embedded_sql_from_source, EmbeddedSqlOptions};

#[test]
fn json_write_projection_does_not_enable_grouping() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/postgres-facts/grouping/fixture");
    let path = root.join("exact-origins.ts");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let source = sources.read_path(&path).unwrap();
    let embedded = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("db", &["query".into()]),
    );
    assert!(!embedded.calls.is_empty());
    let facts = collect(
        &root,
        &sources,
        &[],
        &PostgresSchemaOptions::default(),
        &[embedded],
        false,
        StatementPolicySources {
            collect_json_writes: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!facts.is_empty());
    for fact in facts {
        assert!(!fact.parse_failed);
        assert!(fact.grouping_queries.is_empty());
    }
}
