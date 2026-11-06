use super::*;
use crate::codebase::check_facts::sql_source_prerequisites::{SqlProjection, SqlSourceKey};
use std::path::Path;

const FIXTURE: &str = "helper-tracing-source-prerequisites";

fn prepared_sources() -> (FileIdMap<CheckFileFacts>, PathBuf) {
    let mut files = prepared_fixture(
        FIXTURE,
        &[
            "direct",
            "cached",
            "cached-entry",
            "barrel",
            "reexport",
            "healthy",
            "unused",
        ],
    );
    let broken = files.keys().next().unwrap().with_file_name("broken.mts");
    let source = std::fs::read_to_string(&broken).unwrap();
    let error = crate::ast::with_program(&broken, &source, |_, _| ())
        .expect_err("saved helper is malformed");
    files.insert(
        broken.clone(),
        CheckFileFacts {
            parse_error: Some(error.to_string()),
            source: Some(source.into()),
            ..Default::default()
        },
    );
    (files, broken)
}

fn resolver(files: &FileIdMap<CheckFileFacts>) -> impl Fn(&str, &Path) -> Option<PathBuf> + Sync {
    let paths = files.keys().cloned().collect::<BTreeSet<_>>();
    move |specifier, from| {
        let target = from.parent()?.join(specifier).with_extension("mts");
        paths.contains(&target).then_some(target)
    }
}

#[test]
fn actual_root_consultations_survive_unknown_cache_and_do_not_poison_other_roots() {
    for executed in [false, true] {
        let (mut files, broken) = prepared_sources();
        let resolve = resolver(&files);
        if executed {
            super::super::project_executed(&mut files, resolve);
        } else {
            project(&mut files, resolve);
        }
        for name in ["direct", "cached-entry", "reexport", "healthy", "unused"] {
            let path = broken.with_file_name(format!("{name}.mts"));
            let file = &files[&path];
            for (options, _) in &file.query_annotation {
                let key = SqlSourceKey::profile(SqlProjection::AnnotationMax, options);
                let failures = file
                    .sql_source_prerequisites
                    .failures(&key)
                    .collect::<BTreeSet<_>>();
                let needed = options.import_specifier == "@app/db"
                    && matches!(name, "direct" | "cached-entry" | "reexport");
                assert_eq!(failures.contains(&broken), needed, "{name}, {options:?}");
                assert_eq!(failures.len(), usize::from(needed));
            }
        }
        let healthy = &files[&broken.with_file_name("healthy.mts")];
        assert!(healthy.query_annotation[0]
            .1
            .calls
            .values()
            .any(|sql| sql.as_deref() == Some("SELECT 1")));
    }
}

#[test]
fn recovered_fatal_helper_is_still_a_failed_consulted_source() {
    let mut files = prepared_fixture(
        "helper-tracing-target-reuse",
        &["entry", "barrel", "origin"],
    );
    let origin = files
        .keys()
        .find(|path| path.ends_with("origin.mts"))
        .unwrap()
        .clone();
    Arc::make_mut(&mut files.get_mut(&origin).unwrap().ts).fatal_parse_error = true;
    let resolve = resolver(&files);
    project(&mut files, resolve);
    let entry = &files[&origin.with_file_name("entry.mts")];
    let key = SqlSourceKey::profile(SqlProjection::AnnotationMax, &entry.query_annotation[0].0);
    assert!(entry
        .sql_source_prerequisites
        .failures(&key)
        .any(|path| path == &origin));
}

#[path = "source_profile_scope.rs"]
mod source_profile_scope;
