use super::super as fragments;
use super::*;

fn trusted(options: EmbeddedSqlOptions) -> EmbeddedSqlOptions {
    options.with_trusted_sql_tags(&[crate::codebase::postgres::TrustedSqlTag {
        module: "@example/db".into(),
        name: "sql".into(),
    }])
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
        &trusted(EmbeddedSqlOptions::default()),
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

    let prepared = fragments::collect(&embedded);
    assert_eq!(prepared.len(), 3);
    assert!(!Arc::ptr_eq(
        &prepared[0].statements,
        &prepared[1].statements
    ));
    let has_id_cursor = |fragment: &fragments::PreparedSqlFragment| {
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
        &trusted(EmbeddedSqlOptions::configured("", &["query".to_string()])),
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
    let prepared = fragments::collect(&embedded);
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
    assert!(!fragments::collect(&duplicate_facts)
        .iter()
        .any(|prepared_fragment| prepared_fragment.line == fragment_line));
}
