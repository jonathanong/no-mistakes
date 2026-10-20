use super::*;
use crate::codebase::postgres::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use crate::codebase::rules::postgres_conflict_ordering::{compile_options, Options};

#[test]
fn omitted_non_insert_facts_preserve_valid_and_malformed_policy_verdicts() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/conflict-only-demand-gate.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let embedded = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    let mut malformed = 0;
    for call in &embedded.calls {
        let sql = call.sql_text.as_deref().unwrap();
        if crate::codebase::postgres::may_have_insert_conflict(sql) {
            continue;
        }
        let full = crate::codebase::postgres::extract_sql_statement_facts(sql);
        malformed += usize::from(full.parse_failed);
        for mode in ["fail", "ignore", "advise"] {
            let opts = compile_options(&Options {
                schema_catalog_path: "schema.json".into(),
                unanalyzable_sql: mode.into(),
                ..Default::default()
            })
            .unwrap();
            let catalog = SchemaCatalog::default();
            let before = judge("query.ts", &source, call, Some(&full), &opts, &catalog);
            let after = judge("query.ts", &source, call, None, &opts, &catalog);
            // This is a conflict-policy proof, not a claim that SQL is valid.
            // The existing policy already clears non-INSERT text before parsing.
            assert_eq!(before.status, SqlVerdictStatus::ProvenClear, "{mode}");
            assert_eq!(after.status, before.status, "{mode}");
            assert_eq!(after.reason, before.reason, "{mode}");
            assert_eq!(after.survives_unknown, before.survives_unknown, "{mode}");
        }
    }
    assert_eq!(malformed, 2, "exercise the saved malformed SELECT control");
}
