use super::*;
use crate::codebase::check_facts::CheckFileFacts;

#[test]
fn stale_updates_cannot_create_missing_file_or_profile_facts() {
    let root = crate::test_support::rule_fixture_root("postgres-no-offset")
        .join("low-helper-recovery/src");
    let owner = root.join("query.mts");
    let missing = root.join("absent.mts");
    let broken = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("heads-source-prerequisites/broken.mts");
    let profile = crate::codebase::postgres::EmbeddedSqlOptions::configured("@missing/db", &[]);
    let application = ApplicationKey::new(&profile, "postgres-no-offset", 0);
    let recovered = Arc::new(Recovered::default());
    let mut files = FileIdMap::from_entries([(owner.clone(), CheckFileFacts::default())]);
    apply(
        &mut files,
        vec![
            (
                missing.clone(),
                application.clone(),
                (0, 1),
                recovered.clone(),
            ),
            (owner.clone(), application.clone(), (0, 1), recovered),
        ],
        vec![
            (
                missing.clone(),
                application.clone(),
                (0, 1),
                [broken.clone()].into_iter().collect(),
            ),
            (
                owner.clone(),
                application.clone(),
                (0, 1),
                [broken.clone()].into_iter().collect(),
            ),
        ],
    );
    assert!(files.get(&missing).is_none());
    let file = files.get(&owner).unwrap();
    assert!(file.sql_identifier_syntax.application_calls.is_empty());
    let key = SqlSourceKey::application(
        SqlProjection::LowSyntax,
        &profile,
        &application.rule,
        application.index,
    );
    assert_eq!(
        file.sql_source_prerequisites
            .failures(&key)
            .cloned()
            .collect::<Vec<_>>(),
        vec![broken]
    );
}
