use super::super::ApplicationKey;
use super::failed_source_is_demanded;
use crate::codebase::{check_facts::CheckFileFacts, ts_source::FileIdMap};

#[test]
fn failed_helper_observation_requires_matching_application_demand() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("heads-source-prerequisites/broken.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let error = crate::ast::with_program(&path, &source, |_, _| ()).unwrap_err();
    let profile = super::super::tests::profile();
    let application = ApplicationKey::new(&profile, "postgres-no-offset", 3);
    let mut files = FileIdMap::default();
    assert!(!failed_source_is_demanded(&files, &path, &application));
    files.insert(path.clone(), CheckFileFacts::default());
    assert!(!failed_source_is_demanded(&files, &path, &application));
    files.get_mut(&path).unwrap().parse_error = Some(error.to_string());
    assert!(failed_source_is_demanded(&files, &path, &application));
    files
        .get_mut(&path)
        .unwrap()
        .sql_identifier_syntax
        .low_applications = vec![(profile.clone(), "postgres-no-offset".into(), 2)];
    assert!(!failed_source_is_demanded(&files, &path, &application));
    files
        .get_mut(&path)
        .unwrap()
        .sql_identifier_syntax
        .low_applications
        .push((profile, "postgres-no-offset".into(), 3));
    assert!(failed_source_is_demanded(&files, &path, &application));
}

#[test]
fn declared_empty_demand_survives_failed_source_fact_retention() {
    use crate::codebase::check_facts::{
        sql_source_prerequisites::with_failed_source, CheckFactPlan,
    };
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("heads-source-prerequisites");
    let path = root.join("broken.mts");
    let profile = super::super::tests::profile();
    let application = ApplicationKey::new(&profile, "postgres-no-offset", 0);
    let facts = with_failed_source(
        &root,
        &path,
        &CheckFactPlan::default(),
        CheckFileFacts {
            parse_error: Some("saved source cannot parse".into()),
            ..Default::default()
        },
    );
    let files = FileIdMap::from_entries([(path.clone(), facts)]);
    assert!(!failed_source_is_demanded(&files, &path, &application));
}

#[test]
fn failed_shared_transport_owner_replays_failure_for_both_executor_arguments() {
    let mut files = super::super::patch_support_tests::prepare(&["failed-transport.mts"]);
    let root = super::super::tests::fixture().join("src");
    let path = root.join("failed-transport.mts");
    let helper = root.join("failed-helper.mts");
    let source = std::fs::read_to_string(&helper).unwrap();
    let error = crate::ast::with_program(&helper, &source, |_, _| ()).unwrap_err();
    let profile = super::super::tests::profile();
    let application = ApplicationKey::new(&profile, "postgres-no-offset", 0);
    files
        .get_mut(&path)
        .unwrap()
        .sql_identifier_syntax
        .low_applications = vec![(profile.clone(), "postgres-no-offset".into(), 0)];
    let mut failed = CheckFileFacts {
        parse_error: Some(error.to_string()),
        ..Default::default()
    };
    failed.sql_identifier_syntax.low_applications = vec![(profile, "postgres-no-offset".into(), 0)];
    files.insert(helper.clone(), failed);
    let (updates, failures) = super::project(&files, &|specifier, _| {
        (specifier == "./failed-helper.mjs").then(|| helper.clone())
    });
    assert!(updates.is_empty());
    assert_eq!(failures.len(), 2);
    assert_ne!(failures[0].2, failures[1].2);
    for (owner, key, _, sources) in failures {
        assert_eq!(owner, path);
        assert_eq!(key, application);
        assert_eq!(sources, [helper.clone()].into_iter().collect());
    }
}
