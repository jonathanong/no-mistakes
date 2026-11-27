use super::*;
use std::path::PathBuf;

#[test]
fn legacy_escape_grammar_and_recovered_invalid_raw_text_keep_their_identity() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/ast/js-string-identity");
    let fixture = crate::test_support::materialize_saved_fixture(&root);
    let rows: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fixture.path().join("legacy-raw.json")).unwrap(),
    )
    .unwrap();
    for row in rows.as_array().unwrap() {
        let raw = row["raw"].as_str().unwrap();
        if let Some(needs) = row["needs"].as_bool() {
            assert_eq!(needs_raw_identity(raw), needs, "{raw:?}");
        }
        let text = from_raw(
            raw,
            Cow::Borrowed("fallback😀"),
            row["template"].as_bool().unwrap(),
        );
        assert_eq!(text, row["expected"].as_str().unwrap(), "{raw:?}");
    }
}
