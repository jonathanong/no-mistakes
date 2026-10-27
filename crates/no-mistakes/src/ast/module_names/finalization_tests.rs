use oxc_allocator::Allocator;
use oxc_span::SourceType;
use std::path::PathBuf;

#[test]
fn restored_fatal_finalization_preserves_flow_html_and_eof_lexer_diagnostics() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/ast/js-string-identity/module-names/fatal-finalization");
    let fixture = crate::test_support::materialize_saved_fixture(&root);
    let cases: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fixture.path().join("expected.json")).unwrap(),
    )
    .unwrap();
    for case in cases.as_array().unwrap() {
        let file = case["file"].as_str().unwrap();
        let path = fixture.path().join(file);
        let source = std::fs::read_to_string(&path).unwrap();
        let allocator = Allocator::default();
        let parsed = crate::ast::parse(
            &path,
            &allocator,
            &source,
            SourceType::from_path(&path).unwrap(),
        );
        assert!(parsed.fatal_error, "{file}");
        assert!(parsed.program.body.is_empty(), "{file}");
        assert_eq!(
            parsed.is_flow_language,
            case["flow"].as_bool().unwrap(),
            "{file}"
        );
        let diagnostics: Vec<_> = parsed.diagnostics.iter().map(|diagnostic| {
            serde_json::json!({
                "message": diagnostic.message.as_ref(),
                "labels": diagnostic.labels.iter().map(|label| [label.offset(), label.len()]).collect::<Vec<_>>(),
            })
        }).collect();
        assert_eq!(
            serde_json::json!(diagnostics),
            case["diagnostics"],
            "{file}"
        );
    }
}
