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
    let caller_path = fixture.path().join("main.rs");
    let caller = std::fs::read_to_string(&caller_path).unwrap();
    assert!(caller.contains("include!(\"body.inc\")"));
    assert!(super::rust_sources(fixture.path()).contains(&caller_path));
}
