use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use crate::codebase::postgres::EmbeddedSqlOptions;
use std::{path::PathBuf, sync::Arc};

#[test]
fn fragment_demand_is_explicit_and_equal_text_shares_a_statement_projection() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-shape-policy/fixture/prepared"),
    );
    let file = root.join("src/builders.ts");
    let profile = EmbeddedSqlOptions::default();
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&file));
    let mut plan = CheckFactPlan {
        embedded_sql: true,
        embedded_sql_options: vec![profile.clone()],
        ..Default::default()
    };
    let without = collect_check_facts_with_graph_files_playwright_and_sources(
        &root,
        vec![file.clone()],
        vec![],
        plan.clone(),
        None,
        Arc::clone(&sources),
    );
    assert!(without.postgres_fragments(&file, &profile).is_err());
    plan.include(CheckFactPlan {
        postgres_fragments: true,
        ..Default::default()
    });
    let facts = collect_check_facts_with_graph_files_playwright_and_sources(
        &root,
        vec![file.clone()],
        vec![],
        plan,
        None,
        sources,
    );
    let fragments = facts.postgres_fragments(&file, &profile).unwrap();
    assert_eq!(fragments.len(), 2);
    assert_ne!(fragments[0].line, fragments[1].line);
    assert!(Arc::ptr_eq(
        &fragments[0].statements,
        &fragments[1].statements
    ));
    assert!(std::ptr::eq(
        fragments,
        facts.postgres_fragments(&file, &profile).unwrap()
    ));
    assert!(facts.postgres_statements(&file, Some(&profile)).is_err());
    assert!(facts
        .postgres_fragments(&root.join("absent.ts"), &profile)
        .is_err());
}

#[test]
fn a_clause_only_fragment_keeps_its_limit() {
    for sql in [
        " ORDER BY id LIMIT 500",
        "LIMIT 20",
        " order by id FETCH FIRST 5 ROWS ONLY",
        "OFFSET 10 LIMIT 5",
    ] {
        assert_eq!(super::statement_facts(sql).limit_uses.len(), 1, "{sql}");
    }
    // A predicate fragment is unchanged, and a word that merely starts like a clause is one.
    assert!(super::statement_facts("AND id > $1").limit_uses.is_empty());
    assert!(super::statement_facts("limit_at > $1")
        .limit_uses
        .is_empty());
}
