#[test]
fn run_covers_lazy_import_normal_graph_filters_formats_and_timings() {
    let root = simple_root();

    let mut lazy = traverse_args(root.clone(), vec![PathBuf::from("a.mts")]);
    lazy.relationships = vec![RelationshipArg::Import];
    lazy.format = Some(Format::Md);
    lazy.timings = true;
    run(lazy, Direction::Deps).unwrap();

    let mut normal = traverse_args(root.clone(), vec![PathBuf::from("a.mts")]);
    normal.relationships = vec![RelationshipArg::All];
    normal.filters = vec!["*.mts".to_string()];
    normal.tests = vec!["vitest".to_string()];
    normal.format = Some(Format::Yml);
    run(normal, Direction::Deps).unwrap();

    let mut paths = traverse_args(root, vec![PathBuf::from("a.mts")]);
    paths.format = Some(Format::Paths);
    run(paths, Direction::Deps).unwrap();
}

#[test]
fn run_with_cwd_and_writer_surfaces_output_errors() {
    let observer = crate::diagnostics::InvocationObserver::new(false);
    let _guard = crate::diagnostics::InvocationGuard::install(observer);
    let root = simple_root();
    let args = traverse_args(root, vec![PathBuf::from("a.mts")]);
    let cwd = std::env::current_dir().unwrap();
    let mut out = FailingWriter;
    let mut timings = crate::codebase::timing::PhaseTimings::start();

    let result = collect_and_filter_entries(&args, Direction::Deps, &cwd, &mut timings).unwrap();
    let root_strs: Vec<String> = args.files.iter().map(|f| f.display().to_string()).collect();
    let err = write_output_results(Format::Json, &root_strs, &result, &mut out).unwrap_err();
    timings.mark("output");

    assert!(err.to_string().contains("synthetic write failure"));
    assert!(timings
        .phases
        .iter()
        .any(|(label, _duration)| *label == "output"));
}
