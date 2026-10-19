use super::super::{embedded_call_facts_with_demand, embedded_call_facts_with_locations};
use crate::codebase::postgres::statement_facts::SqlLocationDemand;
use crate::codebase::postgres::{
    extract_embedded_sql_from_source, EmbeddedSqlOptions, TrustedSqlTag,
};

#[test]
fn variants_and_source_mapped_calls_keep_the_complete_projection() {
    for name in [
        "variants-placeholders.ts",
        "source-position-interpolation.ts",
    ] {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/embedded")
            .join(name);
        let source = std::fs::read_to_string(&path).unwrap();
        let file = extract_embedded_sql_from_source(
            &path,
            &source,
            &EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
                TrustedSqlTag {
                    module: "@example/db".into(),
                    name: "sql".into(),
                },
            ]),
        );
        assert!(!file.calls.is_empty());
        assert!(file
            .calls
            .iter()
            .all(|call| !call.variants.is_empty() || !call.sql_source_positions.is_empty()));
        let legacy_full = embedded_call_facts_with_locations(&file, true, false, true);
        let conflict_demand =
            embedded_call_facts_with_demand(&file, true, false, SqlLocationDemand::Conflicts);
        assert_eq!(conflict_demand, legacy_full, "{name}");
        assert!(conflict_demand
            .iter()
            .all(|fact| fact.variant_locations.is_some()));
    }
}
