use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use crate::codebase::postgres::EmbeddedSqlOptions;
use std::path::PathBuf;

#[test]
fn unspecified_and_empty_profile_scans_keep_direct_plan_projections() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres/prepared-projection-scope"),
    );
    let path = root.join("src/a.ts");
    let profile = EmbeddedSqlOptions::configured("@db/a", &["query".into()]);
    for explicit_empty in [false, true] {
        let mut plan = CheckFactPlan {
            embedded_sql: true,
            embedded_sql_options: vec![profile.clone()],
            postgres_dml: true,
            postgres_fragments: true,
            ..Default::default()
        };
        // Library callers without declared rule scans historically request all
        // projections; an empty explicit scan list keeps that compatibility.
        if explicit_empty {
            plan.embedded_sql_rule_ids.insert(profile.clone(), vec![]);
        }
        let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
        let facts = collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            vec![path.clone()],
            vec![],
            plan,
            None,
            sources,
        );
        let statements = facts.postgres_statements(&path, Some(&profile)).unwrap();
        assert_eq!(statements.len(), 1);
        assert!(!statements[0].offset_uses.is_empty());
        assert!(!facts
            .postgres_fragments(&path, &profile)
            .unwrap()
            .is_empty());
    }
}
