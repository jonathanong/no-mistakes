#[test]
fn composed_initializers_map_each_operand_to_its_physical_line() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/source-position-composed.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = super::super::super::extract_embedded_sql_from_source(
        &path,
        &source,
        &super::super::super::EmbeddedSqlOptions::configured("@example/db", &[]),
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
        assert_eq!(call.kind, super::super::super::EmbeddedSqlKind::Composed);
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
    let mut out = super::super::Positions {
        positions: Vec::new(),
        line: 1,
        column: 1,
        origin: 1,
        placeholder_offset: 0,
    };
    assert!(!super::super::compose::try_append(
        expr,
        source,
        expr.span().end as usize,
        1,
        &mut out
    ));
    assert!(out.positions.is_empty());
}
