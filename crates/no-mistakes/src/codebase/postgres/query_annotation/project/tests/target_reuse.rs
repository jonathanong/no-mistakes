use super::*;

#[test]
fn targets_and_indexes_are_shared_across_profiles_and_parallel_evaluators() {
    let mut files = prepared();
    let candidates = files
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<BTreeSet<_>>();
    let calls = Mutex::new(BTreeMap::new());
    let observer = InvocationObserver::new(true);
    with_observer(Some(Arc::clone(&observer)), || {
        project(&mut files, |specifier, from| {
            *calls
                .lock()
                .unwrap()
                .entry((from.to_path_buf(), specifier.to_string()))
                .or_insert(0) += 1;
            let target = from.parent().unwrap().join(specifier).with_extension("mts");
            candidates.contains(&target).then_some(target)
        });
    });
    let calls = calls.into_inner().unwrap();
    assert!(
        calls.len() > 8,
        "exercise the full import/re-export catalog"
    );
    assert!(
        calls.values().all(|count| *count == 1),
        "every source/specifier, including failures, resolves once: {calls:?}"
    );
    assert!(calls
        .keys()
        .any(|(_, specifier)| specifier == "./missing.mjs"));
    // Known executor roots need no speculative resolution of unused imports.
    assert!(!calls
        .keys()
        .any(|(_, specifier)| specifier == "./unused.mjs" || specifier == "./type-only.mjs"));
    let snapshot = observer.snapshot();
    assert_eq!(snapshot.work["postgres.annotation.parallel_passes"], 1);
    assert_eq!(
        snapshot.work["postgres.annotation.modules"],
        candidates.len() as u64
    );
    assert_eq!(
        snapshot.work["postgres.annotation.target_resolutions"],
        calls.len() as u64
    );
    // The same relative string must retain both distinct importing sources.
    for (entry, expected) in [
        ("nested-a/entry.mts", "/* local */ SELECT 5"),
        ("nested-b/entry.mts", "SELECT 6"),
    ] {
        let (path, file) = files
            .iter()
            .find(|(path, _)| path.ends_with(entry))
            .unwrap();
        assert_eq!(calls[&(path.clone(), "./origin.mjs".into())], 1);
        let source = file.source.as_deref().unwrap();
        let call = "query(provide())";
        let start = source.find(call).unwrap() as u32;
        let facts = &file
            .query_annotation
            .iter()
            .find(|(options, _)| options.import_specifier == "@app/db")
            .unwrap()
            .1;
        assert_eq!(
            facts.calls[&(start, start + call.len() as u32)].as_deref(),
            Some(expected)
        );
    }
    let (path, file) = files
        .iter()
        .find(|(path, _)| path.ends_with("helper-tracing-target-reuse/entry.mts"))
        .unwrap();
    let source = file.source.as_deref().unwrap();
    for (options, facts) in &file.query_annotation {
        let executor = if options.import_specifier == "@app/db" {
            "query"
        } else {
            "other"
        };
        let expected = if executor == "query" {
            vec![
                ("safe()", Some("/* safe */ SELECT 1")),
                ("unsafe()", Some("SELECT 1")),
                ("shared()", Some("/* shared */ SELECT 2")),
                ("ambiguous()", None),
                ("missing()", None),
                ("cycled()", None),
                ("absentDefault()", None),
            ]
        } else {
            vec![
                ("safe()", Some("/* safe */ SELECT 1")),
                ("unsafe()", Some("SELECT 1")),
                ("missing()", None),
            ]
        };
        for (argument, expected) in expected {
            let call = format!("{executor}({argument})");
            let start = source.find(&call).unwrap() as u32;
            assert_eq!(
                facts
                    .calls
                    .get(&(start, start + call.len() as u32))
                    .map(|value| value.as_deref()),
                Some(expected),
                "{} {call}",
                path.display()
            );
        }
    }
}

#[test]
fn targets_do_not_escape_their_resolver_projection() {
    let mut files = prepared();
    let candidates = files
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<BTreeSet<_>>();
    let entry = candidates
        .iter()
        .find(|path| path.ends_with("helper-tracing-target-reuse/entry.mts"))
        .unwrap()
        .clone();
    let syntax = Arc::clone(&files[&entry].ts);
    let source = files[&entry].source.as_deref().unwrap();
    let call = "query(safe())";
    let start = source.find(call).unwrap() as u32;
    let span = (start, start + call.len() as u32);
    project(&mut files, |specifier, from| {
        let target = from.parent().unwrap().join(specifier).with_extension("mts");
        candidates.contains(&target).then_some(target)
    });
    assert_eq!(
        files[&entry].query_annotation[0].1.calls[&span].as_deref(),
        Some("/* safe */ SELECT 1")
    );
    let observer = InvocationObserver::new(true);
    // Reproject the exact retained facts. A previous successful target must
    // never answer a query owned by this new resolver/catalog projection.
    with_observer(Some(Arc::clone(&observer)), || {
        project(&mut files, |_, _| None)
    });
    assert!(observer.snapshot().work["postgres.annotation.target_resolutions"] > 0);
    assert_eq!(
        observer.snapshot().work["postgres.annotation.modules"],
        candidates.len() as u64
    );
    assert!(Arc::ptr_eq(&syntax, &files[&entry].ts));
    assert_eq!(files[&entry].query_annotation[0].1.calls[&span], None);
}
