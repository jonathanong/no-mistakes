use super::*;
use crate::codebase::postgres::LENIENT_PARSE_COUNT;

#[test]
fn both_column_kinds_share_one_parse_per_statement() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-generated-column-writes/unit-fixture/both-column-kinds",
    );
    let paths = [root.join("schema.sql"), root.join("writes.sql")];
    let sources = crate::codebase::rules::source_store_for_files(&paths);
    let schema =
        crate::codebase::postgres::extract_schema_facts(&root, &sources, &paths[..1]).unwrap();
    let generated = super::super::catalog::catalog_from_facts(&schema, &[]);
    let trigger =
        super::super::catalog::trigger_catalog_from_facts(&schema, &["updated_at".into()]);
    let mut combined = trigger.clone();
    combined.extend_from(&generated);
    LENIENT_PARSE_COUNT.with(|count| count.set(0));
    let findings = scan_sql_file(&paths[1], "writes.sql", &sources, &combined, &generated);
    assert_eq!(findings.len(), 2);
    LENIENT_PARSE_COUNT.with(|count| assert_eq!(count.get(), 1));
}
