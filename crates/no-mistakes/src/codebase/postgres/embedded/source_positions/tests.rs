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
#[test]
fn high_leading_legacy_octal_keeps_raw_and_cooked_characters_aligned() {
    use oxc_ast::ast::{Expression, Statement};
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/source-position-octal.cjs");
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = oxc_allocator::Allocator::default();
    let parsed = crate::ast::parse(
        &path,
        &allocator,
        &source,
        oxc_span::SourceType::from_path(&path).unwrap(),
    );
    assert_eq!(parsed.program.body.len(), 2);
    for statement in &parsed.program.body {
        let Statement::ExpressionStatement(statement) = statement else {
            panic!("call fixture")
        };
        let Expression::CallExpression(call) = &statement.expression else {
            panic!("call fixture")
        };
        let line =
            crate::codebase::ts_source::byte_offset_to_line(&source, call.span.start as usize);
        let positions = super::for_expression(
            call.arguments[0].as_expression().unwrap(),
            &source,
            call.span.start as usize,
            line,
        );
        assert_eq!(positions.last().unwrap().source_line, line + 1);
    }
    assert_eq!(super::escapes::width("\\400", false), 3);
    assert_eq!(super::escapes::width("\\377", false), 4);
}
#[test]
fn multiline_placeholder_and_following_quasi_have_distinct_physical_lines() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/source-position-interpolation.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = super::super::extract_embedded_sql_from_source(
        &path,
        &source,
        &super::super::EmbeddedSqlOptions::default(),
    );
    let call = &facts.calls[0];
    let sql = call.sql_text.as_deref().unwrap();
    let physical = |word: &str| {
        let column = sql.find(word).unwrap() as u32 + 1;
        call.sql_source_positions
            .iter()
            .rfind(|position| position.sql_column <= column)
            .unwrap()
            .source_line
    };
    assert_eq!(physical("sql_placeholder_1"), 3);
    assert_eq!(physical("NOT IN"), 4);
    assert_eq!(physical("OFFSET"), 4);
}
#[test]
fn bound_static_appends_retain_their_own_physical_origins() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/source-position-append.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = super::super::extract_embedded_sql_from_source(
        &path,
        &source,
        &super::super::EmbeddedSqlOptions::default(),
    );
    assert_eq!(facts.calls.len(), 6);
    assert_eq!(
        facts
            .calls
            .iter()
            .map(|call| call
                .sql_source_positions
                .last()
                .map(|position| position.source_line)
                .unwrap_or(call.declaration_line.unwrap()))
            .collect::<Vec<_>>(),
        [5, 8, 12, 18, 21, 25]
    );
}
