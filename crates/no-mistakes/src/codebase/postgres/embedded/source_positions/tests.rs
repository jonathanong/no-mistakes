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

#[test]
fn composed_initializers_map_each_operand_to_its_physical_line() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/source-position-composed.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = super::super::extract_embedded_sql_from_source(
        &path,
        &source,
        &super::super::EmbeddedSqlOptions::default(),
    );
    assert_eq!(facts.calls.len(), 6);
    for (call, marker) in facts.calls.iter().zip([
        Some("OFFSET 1"),
        Some("OFFSET 2"),
        Some("OFFSET 3"),
        Some("OFFSET 4"),
        None,
        None,
    ]) {
        let Some(marker) = marker else {
            assert!(call.sql_source_positions.is_empty());
            continue;
        };
        assert_eq!(call.kind, super::super::EmbeddedSqlKind::Composed);
        let sql = call.sql_text.as_deref().unwrap();
        let column = sql.find("OFFSET").unwrap() as u32 + 1;
        let origin = call.declaration_line.unwrap_or(call.line);
        let mapped = call
            .sql_source_positions
            .partition_point(|position| (position.sql_line, position.sql_column) <= (1, column))
            .checked_sub(1)
            .map(|index| {
                let position = &call.sql_source_positions[index];
                position.source_line + 1 - position.sql_line
            })
            .unwrap_or(origin);
        let expected = source
            .lines()
            .position(|line| line.contains(marker))
            .unwrap() as u32
            + 1;
        assert_eq!(mapped, expected, "{marker}");
    }
}

#[test]
fn composed_append_rejects_an_operand_before_the_anchor() {
    use oxc_ast::ast::Statement;
    use oxc_span::GetSpan;
    let source = "const q = \"SELECT 1\" +\n  \" OFFSET 2\";\n";
    let path = std::path::Path::new("composed.ts");
    let allocator = oxc_allocator::Allocator::default();
    let parsed = crate::ast::parse(
        path,
        &allocator,
        source,
        oxc_span::SourceType::from_path(path).unwrap(),
    );
    let Statement::VariableDeclaration(declaration) = &parsed.program.body[0] else {
        panic!("decl");
    };
    let expr = declaration.declarations[0].init.as_ref().unwrap();
    let mut out = super::Positions {
        positions: Vec::new(),
        line: 1,
        column: 1,
        origin: 1,
        placeholder_offset: 0,
    };
    assert!(!super::compose::try_append(
        expr,
        source,
        expr.span().end as usize,
        1,
        &mut out
    ));
    assert!(out.positions.is_empty());
}
