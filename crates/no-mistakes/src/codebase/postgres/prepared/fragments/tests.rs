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
    let profile = EmbeddedSqlOptions::configured("@example/db", &[]);
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
        "ORDER\nBY id LIMIT 500",
        "-- page\nORDER BY id LIMIT 500",
        "/* page */ ORDER /* split */ BY id LIMIT 500",
        "LIMIT 20",
        " order by id FETCH FIRST 5 ROWS ONLY",
        "OFFSET 10 LIMIT 5",
    ] {
        assert_eq!(
            super::statement_facts(sql, &[]).limit_uses.len(),
            1,
            "{sql}"
        );
    }
    // A predicate fragment is unchanged, and a word that merely starts like a clause is one.
    assert!(super::statement_facts("AND id > $1", &[])
        .limit_uses
        .is_empty());
    assert!(super::statement_facts("limit_at > $1", &[])
        .limit_uses
        .is_empty());
}

#[test]
fn query_tail_detection_requires_real_keywords() {
    for sql in ["/*", "ORDER wrong", "\"LIMIT\" 5", "5 LIMIT 2"] {
        assert!(!super::starts_with_clause(sql), "{sql}");
    }
}

#[test]
fn builder_fragments_retain_marker_positions_across_cache_keys_and_wrappers() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-sql-shape-policy/fixture/builder-fragment-provenance/src/fragments.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let embedded = crate::codebase::postgres::extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::default(),
    );
    assert_eq!(embedded.fragments.len(), 3);
    assert_eq!(
        embedded.fragments[0].sql_text,
        embedded.fragments[1].sql_text
    );
    assert_ne!(
        embedded.fragments[0].recovered_placeholder_positions,
        embedded.fragments[1].recovered_placeholder_positions
    );

    let prepared = super::collect(&embedded);
    assert_eq!(prepared.len(), 3);
    assert!(!Arc::ptr_eq(
        &prepared[0].statements,
        &prepared[1].statements
    ));
    let has_id_cursor = |fragment: &super::PreparedSqlFragment| {
        fragment.statements.sweeps.iter().any(|sweep| {
            sweep
                .conjuncts
                .iter()
                .any(|conjunct| conjunct.cursor_columns == ["id"])
        })
    };
    assert!(has_id_cursor(&prepared[0]));
    assert!(!has_id_cursor(&prepared[1]));
    assert!(has_id_cursor(&prepared[2]));
}

#[test]
fn executed_sql_suppression_matches_marker_provenance() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-sql-shape-policy/fixture/builder-fragment-provenance/src/executed-fragment.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let embedded = crate::codebase::postgres::extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::default(),
    );
    let executed = embedded
        .calls
        .iter()
        .find(|call| {
            call.sql_text
                .as_deref()
                .is_some_and(|sql| sql.contains("sql_placeholder_1"))
        })
        .expect("fixture query should be recovered");
    let fragment = embedded
        .fragments
        .iter()
        .find(|fragment| {
            fragment.sql_text == executed.sql_text
                && fragment.recovered_placeholder_positions
                    != executed.recovered_placeholder_positions
        })
        .expect("same-text builder fragment should be recovered");
    let fragment_line = fragment.line;
    let prepared = super::collect(&embedded);
    assert!(prepared
        .iter()
        .any(|prepared_fragment| prepared_fragment.line == fragment_line));

    // A call with identical recovered text and positions suppresses the corresponding fragment.
    let mut duplicate_call = executed.clone();
    duplicate_call.sql_text = fragment.sql_text.clone();
    duplicate_call.recovered_placeholder_positions =
        fragment.recovered_placeholder_positions.clone();
    let mut duplicate_facts = embedded.clone();
    duplicate_facts.calls.push(duplicate_call);
    assert!(!super::collect(&duplicate_facts)
        .iter()
        .any(|prepared_fragment| prepared_fragment.line == fragment_line));
}
