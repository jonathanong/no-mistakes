use super::*;

fn entry(pass: FilePass, index: u64, duration: u64) -> FileDiagnostic {
    FileDiagnostic {
        pass,
        path: PathBuf::from(format!("file-{index:02}.ts")),
        duration: Duration::from_nanos(duration),
        units: index,
    }
}
#[test]
fn bounded_reduction_is_independent_of_arrival_order() {
    let passes = [
        FilePass::Parse,
        FilePass::ValueFlowExtract,
        FilePass::ValueFlowFinalize,
        FilePass::PostgresAnnotation,
    ];
    let mut forward = HotFiles::default();
    let mut reverse = HotFiles::default();
    for pass in passes {
        for index in 0..100 {
            forward.record(entry(pass, index, index % 5));
        }
        for index in (0..100).rev() {
            reverse.record(entry(pass, index, index % 5));
        }
    }
    assert_eq!(forward.entries, reverse.entries);
    for entries in forward.entries {
        assert_eq!(entries.len(), 10);
        assert!(entries
            .iter()
            .all(|entry| entry.duration == Duration::from_nanos(4)));
        assert_eq!(entries[0].path, Path::new("file-04.ts"));
        assert_eq!(entries[9].path, Path::new("file-49.ts"));
    }
}
#[test]
fn tie_order_includes_units() {
    let mut files = HotFiles::default();
    let first = entry(FilePass::Parse, 1, 1);
    let mut second = first.clone();
    second.units = 2;
    files.record(first);
    files.record(second.clone());
    assert_eq!(files.entries[0][0], second);
}
#[test]
fn collection_is_verbose_only_and_preserves_result() {
    for verbose in [false, true] {
        let observer = InvocationObserver::new(verbose);
        assert_eq!(
            observer.trace_file(FilePass::Parse, Path::new("file.ts"), 42, || 7),
            7
        );
        assert_eq!(observer.hot_files.is_some(), verbose);
        let block = observer.hot_file_block();
        if verbose {
            assert!(block.starts_with("[hot] parse: file.ts "));
            assert!(block.contains("units=42 (parallel; non-additive)"));
        } else {
            assert!(block.is_empty());
        }
    }
}
#[test]
fn concurrent_writers_keep_only_ten_per_pass() {
    let observer = InvocationObserver::new(true);
    std::thread::scope(|scope| {
        for index in 0..40 {
            let observer = &observer;
            scope.spawn(move || {
                observer.trace_file(
                    FilePass::Parse,
                    &PathBuf::from(format!("file-{index}.ts")),
                    index,
                    || (),
                )
            });
        }
    });
    assert_eq!(
        observer.hot_files.as_ref().unwrap().lock().unwrap().entries[0].len(),
        10
    );
}
#[test]
fn file_scope_restores_after_nested_scope_and_unwind() {
    let observer = InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        with_file(Path::new("outer.ts"), || {
            let _ =
                std::panic::catch_unwind(|| with_file(Path::new("inner.ts"), || panic!("test")));
            trace_current(FilePass::ValueFlowFinalize, 1, || 7);
        });
        trace_current(FilePass::ValueFlowFinalize, 2, || 8);
    });
    let block = observer.hot_file_block();
    assert!(block.contains("outer.ts"));
    assert!(!block.contains("inner.ts"));
    assert_eq!(block.lines().count(), 1);
}
#[test]
fn disabled_scopes_and_traces_preserve_result() {
    crate::diagnostics::with_observer(None, || {
        assert_eq!(
            with_file(Path::new("file.ts"), || trace(
                FilePass::Parse,
                Path::new("file.ts"),
                1,
                || 9
            )),
            9
        );
        assert_eq!(trace_current(FilePass::ValueFlowExtract, 1, || 10), 10);
    });
}

#[test]
fn saved_files_report_physical_parse_extraction_and_finalization() {
    use crate::codebase::ts_source::facts::{
        collect_ts_facts_with_context, TsFactContext, TsFactPlan,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/parser-count/parallel-dedup");
    let fixture = crate::test_support::materialize_saved_fixture(&root);
    let root = fixture.path();
    let files = [root.join("valid.mts"), root.join("syntax-error.mts")];
    let observer = InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        let facts =
            collect_ts_facts_with_context(&files, TsFactPlan::imports(), &TsFactContext::new(root));
        assert_eq!(facts.len(), 2);
    });
    let files = observer.hot_files.as_ref().unwrap().lock().unwrap();
    assert_eq!(files.entries[FilePass::Parse as usize].len(), 2);
    assert_eq!(files.entries[FilePass::ValueFlowExtract as usize].len(), 2);
    assert_eq!(files.entries[FilePass::ValueFlowFinalize as usize].len(), 2);
    assert!(files.entries[FilePass::Parse as usize]
        .iter()
        .all(|entry| entry.units > 0));
}

#[test]
fn repeated_paths_retain_the_largest_operation_without_displacing_other_files() {
    let mut files = HotFiles::default();
    for duration in [5, 3, 7, 2] {
        files.record(entry(FilePass::Parse, 1, duration));
    }
    files.record(entry(FilePass::Parse, 2, 4));
    assert_eq!(files.entries[0].len(), 2);
    assert_eq!(files.entries[0][0], entry(FilePass::Parse, 1, 7));
    assert_eq!(files.entries[0][1], entry(FilePass::Parse, 2, 4));
}

#[test]
fn nested_requests_never_inherit_another_observers_file() {
    use crate::diagnostics::with_observer;
    let outer = InvocationObserver::new(true);
    let inner = InvocationObserver::new(true);
    with_observer(Some(outer.clone()), || {
        with_file(Path::new("outer.ts"), || {
            with_observer(Some(inner.clone()), || {
                trace_current(FilePass::ValueFlowExtract, 1, || ());
                with_file(Path::new("inner.ts"), || {
                    trace_current(FilePass::ValueFlowExtract, 2, || ())
                });
            });
            with_observer(None, || {
                with_file(Path::new("disabled.ts"), || {
                    trace_current(FilePass::ValueFlowExtract, 1, || ())
                });
            });
            trace_current(FilePass::ValueFlowExtract, 3, || ());
        })
    });
    assert_eq!(inner.hot_file_block().lines().count(), 1);
    assert!(inner.hot_file_block().contains("inner.ts"));
    assert!(!inner.hot_file_block().contains("outer.ts"));
    assert_eq!(outer.hot_file_block().lines().count(), 1);
    assert!(outer.hot_file_block().contains("outer.ts"));
}

#[test]
fn rendering_covers_each_pass_in_fixed_order() {
    let observer = InvocationObserver::new(true);
    for pass in [
        FilePass::PostgresAnnotation,
        FilePass::ValueFlowFinalize,
        FilePass::ValueFlowExtract,
        FilePass::Parse,
    ] {
        observer
            .hot_files
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .record(entry(pass, 1, 1_000_000));
    }
    let text = observer.hot_file_block();
    assert_eq!(text, "[hot] parse: file-01.ts 1.000ms units=1 (parallel; non-additive)\n[hot] value_flow.extract: file-01.ts 1.000ms units=1 (parallel; non-additive)\n[hot] value_flow.finalize: file-01.ts 1.000ms units=1 (parallel; non-additive)\n[hot] postgres.annotation: file-01.ts 1.000ms units=1 (parallel; non-additive)\n");
}

#[test]
fn fixed_measurements_render_identically_with_one_or_two_rayon_workers() {
    use rayon::prelude::*;
    let render = |jobs| {
        let observer = InvocationObserver::new(true);
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build()
            .unwrap()
            .install(|| {
                (0..100).into_par_iter().for_each(|index| {
                    observer
                        .hot_files
                        .as_ref()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .record(entry(FilePass::Parse, index, index % 5));
                });
            });
        observer.hot_file_block()
    };
    assert_eq!(render(1), render(2));
}

#[test]
fn disabled_file_cost_skips_the_observer_clock() {
    let observer = InvocationObserver::new(false);
    assert_eq!(
        observer.trace_file_with_clock(
            FilePass::Parse,
            Path::new("file.ts"),
            1,
            || panic!("disabled file diagnostics read the clock"),
            || 7
        ),
        7
    );
    assert!(observer.hot_file_block().is_empty());
}
