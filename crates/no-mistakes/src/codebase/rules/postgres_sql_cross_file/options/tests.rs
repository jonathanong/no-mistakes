#[test]
fn compile_rejects_invalid_include_and_exclude_globs() {
    let opts = super::Options {
        include: vec!["[".into()],
        import_specifier: "@example/db".into(),
        ..Default::default()
    };
    assert!(super::compile("postgres-no-mutable-sql-registry", &opts).is_err());
    let opts = super::Options {
        exclude: vec!["[".into()],
        import_specifier: "@example/db".into(),
        ..Default::default()
    };
    match super::compile("postgres-no-mutable-sql-registry", &opts) {
        Err(error) => assert!(error.to_string().contains("exclude"), "{error}"),
        Ok(_) => panic!("exclude glob should fail"),
    }
}

#[test]
fn includes_honors_exclude_and_empty_include() {
    let compiled = super::compile(
        "postgres-no-mutable-sql-registry",
        &super::Options {
            exclude: vec!["src/skip.ts".into()],
            import_specifier: "@example/db".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(compiled.includes("src/query.ts"));
    assert!(!compiled.includes("src/skip.ts"));
    assert!(!compiled.includes("README.md"));
}
