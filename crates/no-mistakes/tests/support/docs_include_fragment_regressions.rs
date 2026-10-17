use std::path::PathBuf;

#[test]
fn function_body_include_is_an_explicit_fragment_not_a_standalone_rust_unit() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture = super::saved_fixture::materialize_path(
        &root.join("test-cases/rust-validation/fixture/function-body-include"),
    );
    let legacy = std::fs::read_to_string(fixture.path().join("legacy.rs")).unwrap();
    assert!(std::panic::catch_unwind(|| super::subcommand_enums(&legacy)).is_err());
    let fragment = std::fs::read_to_string(fixture.path().join("body.inc")).unwrap();
    assert_eq!(
        legacy, fragment,
        "only the fragment's file identity changes"
    );
    assert!(syn::parse_str::<syn::Expr>(&fragment).is_ok());
    assert!(!super::rust_sources(fixture.path()).contains(&fixture.path().join("body.inc")));

    let source = root.join("crates/no-mistakes/src");
    let directory = source.join("codebase/postgres/query_annotation/evaluate/alternatives");
    let included = directory.join("run_finish.inc");
    assert!(included.is_file() && !directory.join("run_finish.rs").exists());
    let caller = std::fs::read_to_string(directory.join("run.rs")).unwrap();
    assert!(caller.contains("include!(\"run_finish.inc\")"));
    assert!(syn::parse_str::<syn::Expr>(&std::fs::read_to_string(&included).unwrap()).is_ok());
    assert!(!super::rust_sources(&source).contains(&included));
}
