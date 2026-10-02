#[test]
fn saved_escape_encodings_keep_offsets_on_the_physical_source_line() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded");
    let path = root.join("source-position-escapes.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = super::super::extract_embedded_sql_from_source(
        &path,
        &source,
        &super::super::EmbeddedSqlOptions::default(),
    );
    assert_eq!(facts.calls.len(), 7);
    for call in facts.calls {
        let sql = call.sql_text.unwrap();
        let uses = crate::codebase::postgres::sql_file_offset_uses(&sql);
        assert_eq!(uses.len(), 1, "{sql}");
        assert_eq!(
            call.sql_source_positions.first().unwrap().source_line,
            call.line
        );
        assert_eq!(
            call.sql_source_positions.last().unwrap().source_line,
            call.line + 1
        );
    }
}

#[test]
fn physical_line_endings_and_continuations_consume_the_saved_source_width() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/source-position-lines.json");
    let rows: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    for row in rows.as_array().unwrap() {
        let raw = row["raw"].as_str().unwrap();
        assert_eq!(
            super::escapes::width(raw, false),
            row["width"].as_u64().unwrap() as usize
        );
        assert_eq!(
            super::escapes::continuation(raw),
            row["continuation"].as_u64().map(|n| n as usize)
        );
    }
}
