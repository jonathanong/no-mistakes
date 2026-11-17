mod executed;
mod provenance;
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
            super::statement_facts(sql, &[], false).0.limit_uses.len(),
            1,
            "{sql}"
        );
    }
    // A predicate fragment is unchanged, and a word that merely starts like a clause is one.
    assert!(super::statement_facts("AND id > $1", &[], false)
        .0
        .limit_uses
        .is_empty());
    assert!(super::statement_facts("limit_at > $1", &[], false)
        .0
        .limit_uses
        .is_empty());
}

#[test]
fn query_tail_detection_requires_real_keywords() {
    for sql in ["/*", "ORDER wrong", "\"LIMIT\" 5", "5 LIMIT 2"] {
        assert!(!super::starts_with_clause(sql, false), "{sql}");
    }
}

#[test]
fn fragment_origin_metadata_does_not_change_the_public_debug_surface() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-sql-shape-policy/variants/fragment-overlap/src/query.ts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let embedded = crate::codebase::postgres::extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert!(embedded
        .fragment_variants
        .iter()
        .any(|values| !values.is_empty()));
    let mut without_origins = embedded.clone();
    without_origins.fragment_variants.clear();
    assert_eq!(format!("{embedded:?}"), format!("{without_origins:?}"));
    assert_eq!(format!("{embedded:#?}"), format!("{without_origins:#?}"));
}

#[test]
fn finite_where_fragment_projections_keep_a_valid_select_context() {
    let profile = EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
        crate::codebase::postgres::TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        },
    ]);
    for name in ["finite-fragments-good", "finite-fragments-bad"] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../test-cases/rules/postgres-sql-shape-policy/variants/{name}/src/query.ts",
        ));
        let source = std::fs::read_to_string(&path).unwrap();
        let embedded =
            crate::codebase::postgres::extract_embedded_sql_from_source(&path, &source, &profile);
        let prepared = super::collect(&embedded);
        assert_eq!(prepared.len(), 2, "{name}");
        assert!(
            prepared
                .iter()
                .all(|fragment| !fragment.statements.parse_failed),
            "{name}"
        );
        assert_eq!(
            prepared
                .iter()
                .map(|fragment| fragment.statements.function_calls.len())
                .sum::<usize>(),
            usize::from(name.ends_with("bad")),
            "{name}"
        );
    }
}

#[test]
fn balanced_fragment_choices_isolate_the_sixteen_version_cap() {
    let profile = EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
        crate::codebase::postgres::TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        },
    ]);
    for (name, versions) in [
        ("finite-fragments-boundary", 16),
        ("finite-fragments-cap", 0),
    ] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../test-cases/rules/postgres-sql-shape-policy/variants/{name}/src/query.ts",
        ));
        let source = std::fs::read_to_string(&path).unwrap();
        let embedded =
            crate::codebase::postgres::extract_embedded_sql_from_source(&path, &source, &profile);
        assert_eq!(embedded.calls.len(), 1, "{name}");
        let call = &embedded.calls[0];
        assert_eq!(call.variants.len(), versions, "{name}");
        assert_eq!(call.is_unanalyzable(), versions == 0, "{name}");
        assert_eq!(embedded.fragment_variants[0].len(), versions, "{name}");
        assert_eq!(super::collect(&embedded).len(), versions, "{name}");
    }
}
