use super::*;
use crate::codebase::ts_symbols::FileSymbols;
use std::sync::Arc;

#[test]
fn prepared_failure_reasons_are_distinct_and_deterministic() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/queries-incomplete/fixture");
    let files: Vec<_> = [
        "unrelated.ts",
        "broken.ts",
        "consumer.ts",
        "target.ts",
        "target.ts",
    ]
    .iter()
    .map(|file| root.join(file))
    .collect();
    let mut facts = TsFactMap::default();
    facts.insert(
        root.join("consumer.ts"),
        TsFileFacts {
            operational_error: Some("source read failed".into()),
            parse_error: Some("legacy error mirror".into()),
            ..Default::default()
        },
    );
    facts.insert(
        root.join("broken.ts"),
        TsFileFacts {
            parse_error: Some("recovered syntax diagnostic".into()),
            symbols: Some(Arc::new(FileSymbols::default())),
            ..Default::default()
        },
    );
    facts.insert(root.join("target.ts"), TsFileFacts::default());
    let diagnostics = QueryDiagnostics::from_prepared(&files, &facts, &root);
    let json = serde_json::to_value(&diagnostics).unwrap();
    assert_eq!(json["partial"], true);
    assert_eq!(json["incompleteFiles"].as_array().unwrap().len(), 4);
    assert_eq!(json["incompleteFiles"][0]["reason"], "parse-error");
    assert_eq!(json["incompleteFiles"][1]["reason"], "operational-error");
    assert_eq!(json["incompleteFiles"][2]["reason"], "missing-symbols");
    assert_eq!(json["incompleteFiles"][3]["reason"], "missing-facts");
    let mut text = Vec::new();
    diagnostics.write_human(&mut text).unwrap();
    let text = String::from_utf8(text).unwrap();
    assert!(text.contains("Partial analysis"));
    assert!(text.contains("consumer.ts [operational-error]: source read failed"));

    facts.get_mut(&root.join("broken.ts")).unwrap().parse_error = None;
    facts
        .get_mut(&root.join("broken.ts"))
        .unwrap()
        .fatal_parse_error = true;
    let fatal = QueryDiagnostics::from_prepared(&[root.join("broken.ts")], &facts, &root);
    assert_eq!(
        serde_json::to_value(fatal).unwrap()["incompleteFiles"][0]["message"],
        "Fatal parser failure"
    );

    facts
        .get_mut(&root.join("broken.ts"))
        .unwrap()
        .fatal_parse_error = false;
    let success = QueryDiagnostics::from_prepared(&[root.join("broken.ts")], &facts, &root);
    assert!(!success.is_partial());
    assert_eq!(
        serde_json::to_value(&success).unwrap()["incompleteFiles"],
        serde_json::json!([])
    );
    let mut text = Vec::new();
    success.write_human(&mut text).unwrap();
    assert!(text.is_empty());
}

#[test]
fn diagnostic_writes_surface_io_errors() {
    struct Fail;
    impl Write for Fail {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("fixture failure"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let diagnostics = QueryDiagnostics::from_prepared(
        &[PathBuf::from("candidate.ts")],
        &TsFactMap::default(),
        Path::new("."),
    );
    assert!(diagnostics.write_human(&mut Fail).is_err());
}
