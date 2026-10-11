use super::super::embedded_call_facts;
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::{extract_embedded_sql_from_source, EmbeddedSqlOptions};

fn facts() -> (
    String,
    Vec<crate::codebase::postgres::SqlStatementFileFacts>,
) {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/variants-position-facts.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let embedded = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions {
            import_specifier: "@example/db".into(),
            ..Default::default()
        },
    );
    assert_eq!(embedded.calls.len(), 13);
    assert!(embedded.calls[..12]
        .iter()
        .all(|call| call.variants.len() == 2));
    let facts = embedded_call_facts(&embedded, true, false);
    (source, facts)
}
#[test]
fn repeated_mutation_columns_and_merge_clauses_have_distinct_origins() {
    let (source, facts) = facts();
    for fact in facts.iter().filter(|fact| {
        fact.variant_locations
            .as_ref()
            .is_some_and(|locations| locations.call_index == 8)
    }) {
        let locations = fact.variant_locations.as_ref().unwrap();
        assert_eq!(fact.writes.len(), 2);
        let last = locations
            .position(SqlFactSite::WriteColumn(1, "created_at".into()))
            .unwrap();
        assert!(source[last.source_offset.unwrap()..].starts_with("created_at = $2"));
        if let Some(first) = locations.position(SqlFactSite::WriteColumn(0, "created_at".into())) {
            assert_ne!(first.source_offset, last.source_offset);
            assert!(source[first.source_offset.unwrap()..].starts_with("created_at = $1"));
        }
    }
    let merge = facts.iter().find(|fact| fact.variant_locations.as_ref().is_some_and(|locations| locations.call_index == 9) && matches!(&fact.writes[0].columns, crate::codebase::postgres::SqlWriteColumns::Named(columns) if columns.len() == 2)).unwrap();
    let locations = merge.variant_locations.as_ref().unwrap();
    let first = locations
        .position(SqlFactSite::WriteColumn(0, "created_at".into()))
        .unwrap();
    let second = locations
        .position(SqlFactSite::WriteColumn(1, "created_at".into()))
        .unwrap();
    assert_ne!(first.source_offset, second.source_offset);
    assert!(source[first.source_offset.unwrap()..].starts_with("created_at) ="));
    assert!(source[second.source_offset.unwrap()..].starts_with("created_at) VALUES"));
}
#[test]
fn spliced_facts_use_the_branch_token_column_on_one_sql_line() {
    let (source, facts) = facts();
    let grouped = |call: usize| {
        facts.iter().filter(move |fact| {
            fact.variant_locations
                .as_ref()
                .is_some_and(|locations| locations.call_index == call)
        })
    };
    for fact in grouped(0) {
        assert!(matches!(fact.selects[0].relations[0].line, 4 | 5));
        let position = fact
            .variant_locations
            .as_ref()
            .unwrap()
            .position(SqlFactSite::Relation(0, 0))
            .unwrap();
        assert!(source[position.source_offset.unwrap()..].starts_with("accounts"));
    }
    let star = grouped(1)
        .find(|fact| !fact.selects[0].star_projections.is_empty())
        .unwrap();
    assert_eq!(star.selects[0].star_projections[0].line, 8);
    let generated = grouped(2)
        .find(|fact| fact.selects[0].column_uses[0].column == "created_at")
        .unwrap();
    assert_eq!(generated.selects[0].column_uses[0].line, 11);
    let write = grouped(3)
        .find(|fact| {
            fact.variant_locations
                .as_ref()
                .unwrap()
                .position(SqlFactSite::WriteColumn(0, "created_at".into()))
                .is_some()
        })
        .unwrap();
    let position = write
        .variant_locations
        .as_ref()
        .unwrap()
        .position(SqlFactSite::WriteColumn(0, "created_at".into()))
        .unwrap();
    assert_eq!(position.source_line, 14);
    assert!(source[position.source_offset.unwrap()..].starts_with("created_at"));
    let returning = grouped(7)
        .find(|fact| !fact.returning_stars.is_empty())
        .unwrap();
    assert_eq!(returning.returning_stars[0].line, 26);
}
#[test]
fn ordering_is_prepared_with_exact_clause_locations_and_host_identity() {
    let (source, facts) = facts();
    for fact in facts.iter().filter(|fact| {
        fact.variant_locations
            .as_ref()
            .is_some_and(|locations| locations.call_index == 6)
    }) {
        let locations = fact.variant_locations.as_ref().unwrap();
        assert_eq!(locations.original_call_line, 21);
        assert_eq!(locations.locking.len(), 1);
        let position = locations.position(SqlFactSite::Lock(0)).unwrap();
        assert!(matches!(position.source_line, 22 | 23));
        assert!(source[position.source_offset.unwrap()..].starts_with("FOR UPDATE"));
    }
    let conflict = facts
        .iter()
        .find(|fact| {
            fact.variant_locations
                .as_ref()
                .is_some_and(|locations| !locations.conflicts.is_empty())
        })
        .unwrap();
    let locations = conflict.variant_locations.as_ref().unwrap();
    assert!(locations.conflict_error.is_none());
    assert_eq!(locations.conflicts.len(), 1);
    let position = locations.position(SqlFactSite::Conflict(0)).unwrap();
    assert!(source[position.source_offset.unwrap()..].starts_with("ON CONFLICT"));
    let legacy = facts.last().unwrap();
    assert!(legacy.variant_locations.is_none());
    assert!(!format!("{legacy:?}").contains("variant_locations"));
}

#[test]
fn repeated_settings_and_returning_stars_match_their_actual_occurrence() {
    let (source, facts) = facts();
    let returning = facts
        .iter()
        .find(|fact| {
            fact.variant_locations
                .as_ref()
                .is_some_and(|locations| locations.call_index == 10)
                && fact.returning_stars.len() == 2
        })
        .unwrap();
    let locations = returning.variant_locations.as_ref().unwrap();
    let first = locations.position(SqlFactSite::ReturningStar(0)).unwrap();
    let second = locations.position(SqlFactSite::ReturningStar(1)).unwrap();
    assert_ne!(first.source_offset, second.source_offset);
    assert!(source[first.source_offset.unwrap()..].starts_with("*; UPDATE users"));
    assert!(source[second.source_offset.unwrap()..].starts_with("*`"));
    for settings in facts.iter().filter(|fact| {
        fact.variant_locations
            .as_ref()
            .is_some_and(|locations| locations.call_index == 11)
    }) {
        assert_eq!(settings.setting_uses.len(), 2);
        let locations = settings.variant_locations.as_ref().unwrap();
        let first = locations.position(SqlFactSite::Setting(0)).unwrap();
        let second = locations.position(SqlFactSite::Setting(1)).unwrap();
        assert_ne!(first.source_offset, second.source_offset);
        assert!(source[first.source_offset.unwrap()..].starts_with("search_path"));
        assert!(!source[first.source_offset.unwrap()..].starts_with("search_path = $1"));
        assert!(source[second.source_offset.unwrap()..].starts_with("search_path = public`"));
    }
}

#[test]
fn prepared_locking_metadata_preserves_existing_query_traversal() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/variants-lock-traversal.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let embedded = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions {
            import_specifier: "@example/db".into(),
            ..Default::default()
        },
    );
    assert_eq!(embedded.calls.len(), 5);
    let facts = embedded_call_facts(&embedded, false, true);
    assert_eq!(facts.len(), 10);
    for (call_index, call) in embedded.calls.iter().enumerate() {
        for (variant_index, projected) in call.statement_calls().enumerate() {
            let sql = projected.sql_text.as_deref().unwrap();
            let expected =
                crate::codebase::postgres::extract_locking_select_metadata_with_placeholders(
                    sql,
                    &projected.recovered_placeholder_positions,
                )
                .unwrap();
            let locations = facts
                .iter()
                .filter_map(|fact| fact.variant_locations.as_ref())
                .find(|locations| {
                    locations.call_index == call_index && locations.variant_index == variant_index
                })
                .unwrap();
            assert_eq!(locations.locking, expected, "{sql}");
            for index in 0..locations.locking.len() {
                let position = locations.position(SqlFactSite::Lock(index)).unwrap();
                assert!(source[position.source_offset.unwrap()..].starts_with("FOR UPDATE"));
            }
        }
    }
}
