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
fn partial_prepared_sinks_without_executor_metadata_stay_unknown() {
    let mut files = super::super::patch_support_tests::prepare(&["replay.mts"]);
    let path = super::super::tests::fixture().join("src/replay.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let profile = super::super::tests::profile();
    let start = source.find("read(text)").unwrap() as u32;
    let span = (start, start + "read(text)".len() as u32);
    let file = files.get_mut(&path).unwrap();
    file.sql_identifier_syntax.low_applications =
        vec![(profile.clone(), "postgres-no-offset".into(), 0)];
    assert!(file.sql_identifier_syntax.recovery.sinks[&profile].contains(&span));
    file.sql_identifier_syntax.recovery.executors.clear();
    super::project(&mut files, |_, _| None);
    let application = ApplicationKey::new(&profile, "postgres-no-offset", 0);
    let calls = files[&path]
        .sql_identifier_syntax
        .calls_for(&application)
        .unwrap();
    assert!(calls.iter().any(|call| call.span == span
        && matches!(
            call.input,
            crate::codebase::postgres::sql_identifier_syntax::Input::Unknown
        )));
}
