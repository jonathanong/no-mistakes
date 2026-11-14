use crate::codebase::postgres::embedded::tests::{fixture, read_fixture};
use crate::codebase::postgres::embedded::{
    extract_embedded_sql_from_source, EmbeddedSqlOptions, TrustedSqlTag,
};

#[test]
fn early_poison_and_nested_member_escapes_keep_residuals_and_known_mutations() {
    let options =
        EmbeddedSqlOptions::configured("@example/db", &[]).with_raw_sql_tags(&[TrustedSqlTag {
            module: "@example/raw-sql".into(),
            name: "sql".into(),
        }]);
    let source = read_fixture("closed-identifiers-followups.ts");
    let facts = extract_embedded_sql_from_source(
        &fixture("closed-identifiers-followups.ts"),
        &source,
        &options,
    );
    assert_eq!(facts.calls.len(), 10, "{:#?}", facts.calls);
    let mixed = &facts.calls[0];
    assert!(mixed.is_unanalyzable(), "{mixed:#?}");
    assert!(
        mixed
            .variants
            .iter()
            .any(|variant| variant.sql_text == "DROP TABLE users" && !variant.variants_truncated),
        "{mixed:#?}"
    );
    for i in [1, 2, 5, 6, 7, 8] {
        assert!(facts.calls[i].is_unanalyzable(), "{:#?}", facts.calls[i]);
    }
    for i in [3, 4] {
        assert!(!facts.calls[i].is_unanalyzable(), "{:#?}", facts.calls[i]);
        assert_eq!(facts.calls[i].variants.len(), 2);
    }
    let drop = facts.calls[9]
        .variants
        .iter()
        .find(|variant| variant.sql_text == "DROP TABLE users")
        .unwrap();
    let return_line = source
        .lines()
        .position(|line| line.contains("case 'drop': return"))
        .unwrap()
        + 1;
    assert_eq!(drop.line as usize, return_line, "{drop:#?}");
}
