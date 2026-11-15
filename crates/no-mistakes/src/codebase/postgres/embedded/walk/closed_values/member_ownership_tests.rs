use crate::codebase::postgres::embedded::tests::{fixture, read_fixture};
use crate::codebase::postgres::embedded::{
    extract_embedded_sql_from_source, EmbeddedSqlOptions, TrustedSqlTag,
};

#[test]
fn untracked_member_results_open_the_owner_while_nested_scalar_reads_stay_closed() {
    let options =
        EmbeddedSqlOptions::configured("@example/db", &[]).with_raw_sql_tags(&[TrustedSqlTag {
            module: "@example/raw-sql".into(),
            name: "sql".into(),
        }]);
    let facts = extract_embedded_sql_from_source(
        &fixture("closed-identifiers-member-ownership.tsx"),
        &read_fixture("closed-identifiers-member-ownership.tsx"),
        &options,
    );
    assert_eq!(facts.calls.len(), 9, "{:#?}", facts.calls);
    for i in [0, 1, 2, 3, 4, 8] {
        assert!(facts.calls[i].is_unanalyzable(), "{:#?}", facts.calls[i]);
        assert!(facts.calls[i]
            .variants
            .iter()
            .all(|variant| variant.variants_truncated));
    }
    for i in [5, 6, 7] {
        assert!(!facts.calls[i].is_unanalyzable(), "{:#?}", facts.calls[i]);
        assert_eq!(facts.calls[i].variants.len(), 2);
    }
}
