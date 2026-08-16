#[test]
fn language_frontend_edges_keep_go_imports_across_modules() {
    let go = lang_fixture("go-asynq");
    let mut options = lang_options();
    options.go_modules = vec!["worker".into(), "nested".into()];
    let go_edges = collect_language_frontend_edges(&go, &lang_files(&go), Some(&options));
    assert!(go_edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::GoImport
            && from
                .as_file()
                .is_some_and(|path| path.ends_with("enqueue.go"))
            && to.as_file().is_some_and(|path| path.ends_with("nested/mail.go"))
    }));
}
