use super::*;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_span::SourceType;
use std::path::{Path, PathBuf};

#[test]
fn ordinary_module_property_and_member_forms_borrow_or_preserve_absence() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/ast/js-string-identity");
    let fixture = crate::test_support::materialize_saved_fixture(&root);
    let source = std::fs::read_to_string(fixture.path().join("helper-controls.ts")).unwrap();
    let metadata: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fixture.path().join("synthetic-literal.json")).unwrap(),
    )
    .unwrap();
    let allocator = Allocator::default();
    let mut parsed = crate::ast::parse(
        Path::new("helper-controls.ts"),
        &allocator,
        &source,
        SourceType::ts(),
    );
    assert!(parsed.diagnostics.is_empty());
    let Statement::ExportAllDeclaration(export) = &parsed.program.body[1] else {
        panic!("expected export")
    };
    assert!(matches!(
        module_export_name(export.exported.as_ref().unwrap()),
        Cow::Borrowed("ordinary")
    ));
    let Statement::VariableDeclaration(declaration) = &parsed.program.body[2] else {
        panic!("expected declaration")
    };
    let Some(Expression::ObjectExpression(object)) = &declaration.declarations[0].init else {
        panic!("expected object")
    };
    assert!(matches!(
        property_key_name(&object.properties[0].as_property().unwrap().key),
        Some(Cow::Borrowed("plain"))
    ));
    let mut names = Vec::new();
    for statement in &parsed.program.body[3..7] {
        let Statement::ExpressionStatement(statement) = statement else {
            panic!("expected expression")
        };
        names.push(
            member_property_name(statement.expression.as_member_expression().unwrap())
                .map(Cow::into_owned),
        );
    }
    assert_eq!(
        names,
        [
            Some("/raw/".into()),
            None,
            Some("ordinary".into()),
            Some("plain".into())
        ]
    );
    let Statement::ExpressionStatement(statement) = &mut parsed.program.body[7] else {
        panic!("expected expression")
    };
    let Expression::StringLiteral(literal) = &mut statement.expression else {
        panic!("expected literal")
    };
    // Caller-built AST metadata can lack quotes or start inside a UTF-8
    // character. Keep the complete cooked identity when raw metadata is invalid.
    literal.raw = Some(oxc_str::Str::from(
        metadata["malformedRaw"].as_str().unwrap(),
    ));
    assert_eq!(js_literal(literal), "escaped\n");
    // Synthetic AST literals may also omit raw metadata entirely.
    literal.raw = None;
    assert_eq!(js_literal(literal), "escaped\n");
}
