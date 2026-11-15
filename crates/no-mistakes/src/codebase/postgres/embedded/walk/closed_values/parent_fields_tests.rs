use crate::codebase::postgres::embedded::tests::{fixture, read_fixture};
use crate::codebase::postgres::embedded::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use crate::codebase::postgres::{
    extract_executed_postgres_sql_with_imports, ExecutedPostgresSource,
};

#[test]
fn proven_singletons_sync_parent_fields_without_promoting_unknown_residuals() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let source = read_fixture("closed-identifiers-parent-fields.ts");
    let facts = extract_embedded_sql_from_source(
        &fixture("closed-identifiers-parent-fields.ts"),
        &source,
        &options,
    );
    assert_eq!(facts.calls.len(), 3);
    for call in &facts.calls[..2] {
        assert!(!call.is_unanalyzable(), "{call:#?}");
        let [variant] = call.variants.as_slice() else {
            panic!("{call:#?}")
        };
        assert_eq!(
            call.sql_text.as_deref(),
            Some("SELECT id FROM users WHERE id = $1")
        );
        assert_eq!(call.sql_text.as_deref(), Some(variant.sql_text.as_str()));
        assert_eq!(call.sql_source_positions, variant.sql_source_positions);
        assert_eq!(call.sql_source_offsets, variant.sql_source_offsets);
        assert_eq!(
            call.recovered_placeholder_positions,
            variant.recovered_placeholder_positions
        );
        assert_eq!(call.bound_parameters.len(), 1);
        assert!(call.bound_parameters[0].is_some());
    }
    assert!(facts.calls[2].is_unanalyzable(), "{:#?}", facts.calls[2]);
    assert!(facts.calls[2].variants_truncated());
}

#[test]
fn exported_flat_maps_keep_the_legacy_precise_record_projection() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let source = read_fixture("closed-identifiers-exported-flat.ts");
    let path = fixture("closed-identifiers-exported-flat.ts");
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert_eq!(facts.calls.len(), 3);
    for call in &facts.calls[..2] {
        assert!(call.is_unanalyzable(), "{call:#?}");
        assert!(!call.variants_truncated(), "{call:#?}");
    }
    assert!(facts.calls[2].is_unanalyzable());
    assert!(facts.calls[2].variants_truncated());
    let projected = extract_executed_postgres_sql_with_imports(
        &[ExecutedPostgresSource {
            file_name: path.to_string_lossy().into_owned(),
            source,
        }],
        &options,
        &[],
    )
    .unwrap();
    for (sql, expected) in projected.iter().take(2).zip(["app.users", "app.posts"]) {
        assert!(sql.complete, "{sql:#?}");
        assert!(
            sql.sql_text.as_deref().unwrap().contains(expected),
            "{sql:#?}"
        );
    }
    assert!(!projected[2].complete, "{:#?}", projected[2]);
}
