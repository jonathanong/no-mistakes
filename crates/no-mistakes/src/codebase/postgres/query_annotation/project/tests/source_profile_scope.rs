use super::*;

#[test]
fn failed_reexport_provider_keeps_its_declared_profile_catalog_membership() {
    for member in [false, true] {
        let mut files = prepared_fixture(
            "helper-tracing-target-reuse",
            &["entry", "barrel", "origin"],
        );
        let origin = files
            .keys()
            .find(|path| path.ends_with("origin.mts"))
            .unwrap()
            .clone();
        for (path, file) in &mut files {
            let wanted = if path == &origin && !member {
                "@app/other"
            } else {
                "@app/db"
            };
            file.query_annotation
                .retain(|(profile, _)| profile.import_specifier == wanted);
            file.embedded_sql
                .retain(|(profile, _)| profile.import_specifier == wanted);
        }
        Arc::make_mut(&mut files.get_mut(&origin).unwrap().ts).fatal_parse_error = true;
        let resolve = resolver(&files);
        project(&mut files, resolve);
        let entry = &files[&origin.with_file_name("entry.mts")];
        let key = SqlSourceKey::profile(SqlProjection::AnnotationMax, &entry.query_annotation[0].0);
        assert_eq!(
            entry
                .sql_source_prerequisites
                .failures(&key)
                .any(|path| path == &origin),
            member,
            "provider membership={member}"
        );
    }
}

#[test]
fn failure_free_profiles_share_the_empty_source_lookup() {
    let files = prepared_fixture(
        "helper-tracing-profile-lifetimes",
        &["fast", "slow-entry", "slow"],
    );
    let resolve = resolver(&files);
    let profiles = super::super::super::prepare::profiles(&files, &resolve);
    assert_eq!(profiles.len(), 2);
    assert!(profiles
        .iter()
        .all(|profile| profile.failed_sources.is_empty()));
    assert!(Arc::ptr_eq(
        &profiles[0].failed_sources,
        &profiles[1].failed_sources
    ));
}
