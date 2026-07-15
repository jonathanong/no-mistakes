use super::*;
use std::path::PathBuf;

#[test]
fn parser_reports_invalid_sources_and_extensions() {
    assert!(with_program(Path::new("fixture.txt"), "", |_, _| ())
        .err()
        .unwrap()
        .to_string()
        .contains("unsupported"));

    assert!(
        with_program(Path::new("fixture.ts"), "await page.goto(", |_, _| ())
            .err()
            .unwrap()
            .to_string()
            .contains("failed to parse")
    );

    let _ = with_program(Path::new("non-existent.ts"), "", |_, _| ());
}

#[test]
fn parser_chokepoint_observes_successes_and_failures() {
    let root = PathBuf::from("parser-observation");
    let valid = root.join("valid.ts");
    let invalid = root.join("invalid.ts");
    begin_parse_count(&root);

    assert!(with_program(&valid, "export const value = 1;", |_, _| ()).is_ok());
    assert!(with_program(&invalid, "export const =", |_, _| ()).is_err());

    let counts = finish_parse_count(&root);
    assert_eq!(counts.get(&valid), Some(&1));
    assert_eq!(counts.get(&invalid), Some(&1));
}

#[test]
fn parser_chokepoint_observes_synthetic_parses_from_rayon_workers() {
    let root = PathBuf::from("rayon-parser-observation");
    let sentinel = root.join("source-only-compatibility.ts");
    begin_parse_count(&root);

    rayon::scope(|scope| {
        scope.spawn(|_| {
            let allocator = Allocator::default();
            let parsed = parse(&sentinel, &allocator, "export {};", SourceType::ts());
            assert!(parsed.diagnostics.is_empty());
        });
    });

    let counts = finish_parse_count(&root);
    assert_eq!(counts.get(&sentinel), Some(&1));
}

#[test]
fn production_oxc_parses_use_the_observable_chokepoint() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&src);
    let sources = snapshot.source_store_for(&src);
    let offenders = sources
        .inventory()
        .paths()
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .filter(|path| {
            let relative = path.strip_prefix(&src).unwrap();
            relative != Path::new("ast.rs")
                && !relative
                    .components()
                    .any(|component| component.as_os_str() == "tests")
                && !relative
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with("_tests.rs") || name == "tests.rs")
        })
        .filter_map(|path| {
            let source = sources.read_path(path).ok()?;
            let lines = source.lines().collect::<Vec<_>>();
            let has_production_reference = lines.iter().enumerate().any(|(index, line)| {
                let references_parser =
                    line.contains("oxc_parser") || line.contains("Parser::new(");
                let test_only_import = index > 0
                    && lines[index - 1].trim() == "#[cfg(test)]"
                    && line.trim_start().starts_with("use ");
                references_parser && !test_only_import
            });
            has_production_reference.then(|| path.strip_prefix(&src).unwrap().to_path_buf())
        })
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        "production OXC parser entrypoints must call crate::ast::parse: {offenders:?}"
    );
}

#[test]
fn parsed_program_cache_reuses_parse_and_source_type_errors() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/integration-tests/parse-errors/fixture");
    let syntax_error_path = root.join("vitest.syntax-error.mts");
    let syntax_error = std::fs::read_to_string(&syntax_error_path).unwrap();
    let cache = ParsedProgramCache::default();

    let first = cache
        .with_program(&syntax_error_path, &syntax_error, |_, _| ())
        .unwrap_err();
    let cached = cache
        .with_program(&syntax_error_path, "export default {}", |_, _| ())
        .unwrap_err();
    assert_eq!(cached, first, "a request cache is keyed by normalized path");

    let unsupported_path = root.join("../README.md");
    let first = cache
        .with_program(&unsupported_path, "", |_, _| ())
        .unwrap_err();
    let cached = cache
        .with_program(&unsupported_path, "", |_, _| ())
        .unwrap_err();
    assert_eq!(cached, first);
    assert!(first.contains("unsupported JavaScript/TypeScript file"));
}

#[test]
fn parsed_program_cache_clear_releases_cached_programs() {
    let cache = ParsedProgramCache::default();
    let path = Path::new("fixture.ts");
    cache
        .with_program(path, "export default {};", |_, _| ())
        .unwrap();
    cache.clear();
    assert!(cache
        .with_program(path, "export default (", |_, _| ())
        .is_err());
}

#[test]
fn test_span_text() {
    assert_eq!(span_text("abc", Span::new(0, 3)), "abc");
    assert_eq!(span_text("abc", Span::new(0, 0)), "");
    assert_eq!(span_text("abc", Span::new(9, 10)), "");
}

fn statement_expression<'a>(statement: &'a oxc_ast::ast::Statement<'a>) -> &'a Expression<'a> {
    let oxc_ast::ast::Statement::ExpressionStatement(expr) = statement else {
        panic!("expected expression statement");
    };
    &expr.expression
}

#[test]
#[should_panic]
fn test_statement_expression_panics_when_not_expression_statement() {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(Path::new("test.ts")).unwrap();
    let source = "if (true) {}";
    let parsed = Parser::new(&allocator, source, source_type).parse();
    statement_expression(&parsed.program.body[0]);
}

fn statement_template_literal<'a>(
    statement: &'a oxc_ast::ast::Statement<'a>,
) -> &'a TemplateLiteral<'a> {
    let oxc_ast::ast::Expression::TemplateLiteral(template) = statement_expression(statement)
    else {
        panic!("expected template literal");
    };
    template
}

#[test]
#[should_panic]
fn test_statement_template_literal_panics_when_not_template_literal() {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(Path::new("test.ts")).unwrap();
    let source = "1 + 1;";
    let parsed = Parser::new(&allocator, source, source_type).parse();
    statement_template_literal(&parsed.program.body[0]);
}

#[test]
fn test_template_literal_text() {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(Path::new("test.ts")).unwrap();
    let source = "`${a}b${c}`";
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "template literal parse errors: {:?}",
        parsed.diagnostics
    );
    let t = statement_template_literal(&parsed.program.body[0]);
    assert_eq!(template_literal_text(t, source), "${a}b${c}");

    let source = "`no_expressions`";
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "template literal parse errors: {:?}",
        parsed.diagnostics
    );
    let t = statement_template_literal(&parsed.program.body[0]);
    assert_eq!(template_literal_text(t, source), "no_expressions");
}

#[test]
fn test_binary_concat_path_text() {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(Path::new("test.ts")).unwrap();
    let source = r#""/api/" + ("v1/") + `${resource}/` + id"#;
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "parse errors: {:?}",
        parsed.diagnostics
    );
    let Expression::BinaryExpression(binary) = statement_expression(&parsed.program.body[0]) else {
        panic!("expected binary expression");
    };
    assert_eq!(
        binary_concat_path_text(binary, source).as_deref(),
        Some("/api/v1/${resource}/${id}")
    );

    let source = "1 - 1";
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "parse errors: {:?}",
        parsed.diagnostics
    );
    let Expression::BinaryExpression(binary) = statement_expression(&parsed.program.body[0]) else {
        panic!("expected binary expression");
    };
    assert_eq!(binary_concat_path_text(binary, source), None);
}

#[test]
fn test_expression_path() {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(Path::new("test.ts")).unwrap();

    let source = "a.b.c";
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "parse errors: {:?}",
        parsed.diagnostics
    );
    let path = expression_path(statement_expression(&parsed.program.body[0])).unwrap();
    assert_eq!(path, vec!["a", "b", "c"]);

    let source = "(a).b";
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "parse errors: {:?}",
        parsed.diagnostics
    );
    let path = expression_path(statement_expression(&parsed.program.body[0])).unwrap();
    assert_eq!(path, vec!["a", "b"]);

    let source = "123";
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "parse errors: {:?}",
        parsed.diagnostics
    );
    assert_eq!(
        expression_path(statement_expression(&parsed.program.body[0])),
        None
    );

    let source = "a['b']";
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "parse errors: {:?}",
        parsed.diagnostics
    );
    assert_eq!(
        expression_path(statement_expression(&parsed.program.body[0])),
        None
    );
}

pub(crate) fn request_parse_cache_len() -> usize {
    super::current_request_parse_cache().map_or(0, |cache| super::parsed_cache::tests::len(&cache))
}
