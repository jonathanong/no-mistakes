use super::*;
use crate::codebase::check_facts::sql_source_prerequisites::{SqlProjection, SqlSourceKey};

#[test]
fn contextual_imported_calls_retain_only_their_actual_root_prerequisites() {
    for executed in [false, true] {
        let mut files = prepared_fixture(
            "helper-tracing-event-prerequisites",
            &["sink", "healthy-sink", "failed-caller", "healthy-caller"],
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
        let paths = files.keys().cloned().collect::<BTreeSet<_>>();
        let resolve = move |specifier: &str, from: &std::path::Path| {
            let target = from.parent()?.join(specifier).with_extension("mts");
            paths.contains(&target).then_some(target)
        };
        if executed {
            super::super::project_executed(&mut files, resolve);
        } else {
            project(&mut files, resolve);
        }
        for name in ["sink", "healthy-sink"] {
            let file = &files[&broken.with_file_name(format!("{name}.mts"))];
            for (options, _) in &file.query_annotation {
                let key = SqlSourceKey::profile(SqlProjection::AnnotationMax, options);
                let failures = file
                    .sql_source_prerequisites
                    .failures(&key)
                    .collect::<BTreeSet<_>>();
                let needed = name == "sink" && options.import_specifier == "@app/db";
                assert_eq!(
                    failures.contains(&broken),
                    needed,
                    "{name}, {options:?}, executed={executed}"
                );
                assert_eq!(failures.len(), usize::from(needed));
            }
        }
    }
}

#[test]
fn discarded_generic_events_do_not_add_contextual_fact_prerequisites() {
    use super::super::merge;
    use crate::codebase::postgres::query_annotation::evaluate::{ExecutionEvents, Value};
    for executed in [false, true] {
        let mut files = prepared_fixture("helper-tracing-event-prerequisites", &["sink"]);
        let path = files.keys().next().unwrap().clone();
        let broken = path.with_file_name("broken.mts");
        assert!(broken.is_file());
        let file = &files[&path];
        let profile = file.query_annotation[0].0.clone();
        let span = file.embedded_sql[0].1.call_spans[0];
        let events = ExecutionEvents::from([(
            (path.clone(), span),
            vec![(true, Value::Unknown), (false, Value::Unknown)],
        )]);
        let sources = merge::SourceEvents::from([(
            (path.clone(), span),
            vec![(true, [broken.clone()].into_iter().collect())],
        )]);
        let projection = merge::reduce_with_sources(events, executed, sources);
        let originals = [(path.clone(), path.clone())].into_iter().collect();
        merge::apply(
            &mut files,
            &originals,
            &profile,
            projection,
            &mut Default::default(),
        );
        let key = SqlSourceKey::profile(SqlProjection::AnnotationMax, &profile);
        assert_eq!(
            files[&path]
                .sql_source_prerequisites
                .failures(&key)
                .any(|source| source == &broken),
            executed
        );
    }
}
