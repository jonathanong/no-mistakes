use super::*;

#[test]
fn demand_prunes_unrelated_modules_and_cycles_but_keeps_contextual_callers() {
    let mut files = prepared_fixture(
        "helper-tracing-demand-roots",
        &[
            "executor",
            "caller",
            "barrel",
            "entry",
            "cycle-a",
            "cycle-b",
            "unrelated",
            "unrelated-chain",
            "unrelated-cycle-a",
            "unrelated-cycle-b",
            "unresolved",
            "opaque-consumer",
            "computed",
            "dynamic",
            "required",
            "type-only",
        ],
    );
    for (path, file) in &mut files {
        file.query_annotation.truncate(1);
        file.embedded_sql.truncate(1);
        // entry.mts only forwards SQL. Rejecting the file keeps it as a root.
        if path.ends_with("entry.mts") {
            file.query_annotation[0].1.annotation_event = false;
        }
    }
    let candidates = files.keys().cloned().collect::<BTreeSet<_>>();
    let resolve = |specifier: &str, from: &std::path::Path| {
        let target = from.parent().unwrap().join(specifier).with_extension("mts");
        candidates.contains(&target).then_some(target)
    };
    let collected = files
        .iter()
        .map(|(path, file)| {
            (
                path.clone(),
                super::super::File {
                    module: Arc::new(super::super::Module::new(Arc::clone(&file.ts))),
                    facts: &file.query_annotation[0].1,
                    executors: file.embedded_sql[0].1.call_spans.iter().copied().collect(),
                },
            )
        })
        .collect();
    let selected = super::super::demand::roots(&collected, &resolve)
        .into_iter()
        .map(|path| path.file_stem().unwrap().to_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        selected,
        BTreeSet::from([
            "executor", "caller", "barrel", "entry", "cycle-a", "cycle-b", "required",
        ])
    );
    drop(collected);
    let observer = InvocationObserver::new(true);
    let resolutions = Mutex::new(BTreeSet::new());
    with_observer(Some(Arc::clone(&observer)), || {
        project(&mut files, |specifier, from| {
            resolutions
                .lock()
                .unwrap()
                .insert((from.to_path_buf(), specifier.to_string()));
            let target = from.parent().unwrap().join(specifier).with_extension("mts");
            candidates.contains(&target).then_some(target)
        });
    });
    assert_eq!(observer.snapshot().work["postgres.annotation.roots"], 7);
    assert!(
        resolutions
            .into_inner()
            .unwrap()
            .iter()
            .any(|(from, specifier)| {
                from.file_stem().unwrap() == "unresolved" && specifier == "./unrelated.mjs"
            }),
        "a missing target must not hide a later reachable runtime edge"
    );
    let executor = files
        .iter()
        .find(|(path, _)| path.ends_with("executor.mts"))
        .unwrap()
        .1;
    let source = executor.source.as_deref().unwrap();
    let start = source.find("query(sql)").unwrap() as u32;
    // Executor-free importers must supply their contextual unannotated SQL.
    assert_eq!(
        executor.query_annotation[0].1.calls[&(start, start + 10)].as_deref(),
        Some("SELECT 1")
    );

    // An excluded barrel re-exports the executor. Demand must still reach the
    // included caller on the other side of that barrel.
    for (path, file) in &mut files {
        if path.ends_with("entry.mts") {
            file.query_annotation[0].1.annotation_event = true;
        }
        if path.ends_with("barrel.mts") {
            file.query_annotation[0].1.annotation_event = false;
        }
    }
    let collected = files
        .iter()
        .map(|(path, file)| {
            (
                path.clone(),
                super::super::File {
                    module: Arc::new(super::super::Module::new(Arc::clone(&file.ts))),
                    facts: &file.query_annotation[0].1,
                    executors: file.embedded_sql[0].1.call_spans.iter().copied().collect(),
                },
            )
        })
        .collect();
    let selected = super::super::demand::roots(&collected, &resolve)
        .into_iter()
        .map(|path| path.file_stem().unwrap().to_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert!(
        selected.contains("entry"),
        "included caller of an excluded re-export"
    );
    assert!(
        selected.contains("barrel"),
        "excluded re-export still forwards demand"
    );
    assert!(selected.contains("executor"));
}
