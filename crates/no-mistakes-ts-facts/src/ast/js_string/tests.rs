use super::*;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_span::SourceType;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/ast/js-string-identity");
    let saved = crate::test_support::materialize_saved_fixture(&root);
    std::fs::read_to_string(saved.path().join(name)).unwrap()
}

#[test]
fn utf8_identity_borrows_original_storage() {
    for value in ["", "ascii", "café 世界", "�", "😀", "\\ud800", "\0"] {
        let text = js_string(JSStr::from(value));
        assert!(matches!(text, Cow::Borrowed(_)), "{value:?}");
        assert_eq!(text.as_ptr(), value.as_ptr());
        assert_eq!(text, value);
    }
}

#[test]
fn parsed_literals_and_templates_preserve_legacy_identity_and_raw_spans() {
    let source = fixture("values.ts");
    let allocator = Allocator::default();
    let parsed = crate::ast::parse(
        std::path::Path::new("values.ts"),
        &allocator,
        &source,
        SourceType::ts(),
    );
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::VariableDeclaration(declaration) = &parsed.program.body[0] else {
        panic!("expected variable declaration");
    };
    let Some(Expression::ArrayExpression(array)) = &declaration.declarations[0].init else {
        panic!("expected array");
    };
    let expected = [
        "plain",
        "café 世界",
        "�",
        "�",
        "😀",
        "😀",
        "�d800",
        "�dc00",
        "�dfff",
        "�d800",
        "�fffd�d800�fffd",
        "�fffd�d800�fffd",
        "😀�d800x�dc00",
        "plain",
        "café 世界",
        "�",
        "�",
        "😀",
        "😀",
        "�d800",
        "�dc00",
        "�fffd�d800�fffd",
        "�fffd�d800�fffd",
        "�d800${value}�",
        "�${value}�d800",
        "\\u{invalid}",
        "�d800�dc00",
        "�d800�dc00",
        "�d800�dc00",
        "�fffd�d800�dc00�fffd",
        "�d800😀",
        "�d800�dc00",
        "�d800�dc00",
        "�d800�dc00",
        "�d800�dc00",
        "�fffd�d800�dc00�fffd",
        "�d800�dc00",
        "a\n�d800�dc00\t'\\",
        "😀",
        "\\u{D800}�dc00",
    ];
    let mut actual = Vec::new();
    for element in &array.elements {
        let expression = element.as_expression().expect("fixture expression");
        match expression {
            Expression::StringLiteral(literal) => {
                let raw = crate::ast::span_text(&source, literal.span);
                assert_eq!(literal.raw.as_ref().unwrap().as_str(), raw);
                let text = js_literal(literal);
                if literal.value.has_lone_surrogate() {
                    assert!(matches!(text, Cow::Owned(_)));
                }
                actual.push(text.into_owned());
            }
            Expression::TemplateLiteral(template) => {
                for quasi in &template.quasis {
                    assert_eq!(
                        quasi.value.raw.as_str(),
                        crate::ast::span_text(&source, quasi.span)
                    );
                }
                actual.push(crate::ast::template_literal_text(template, &source));
            }
            Expression::TaggedTemplateExpression(tagged) => {
                assert!(tagged.quasi.quasis[0].value.cooked.is_none());
                actual.push(crate::ast::template_literal_text(&tagged.quasi, &source));
            }
            _ => panic!("unexpected fixture expression"),
        }
    }
    assert_eq!(actual, expected);
}

#[test]
fn module_names_and_static_properties_keep_their_identity() {
    let source = fixture("recovered.ts");
    let allocator = Allocator::default();
    let parsed = crate::ast::parse(
        std::path::Path::new("recovered.ts"),
        &allocator,
        &source,
        SourceType::ts(),
    );
    assert!(parsed.diagnostics.is_empty());
    let Statement::ExportNamedDeclaration(export) = &parsed.program.body[0] else {
        panic!("expected export");
    };
    assert_eq!(module_export_name(&export.specifiers[0].local), "value");
    assert_eq!(
        module_export_name(&export.specifiers[0].exported),
        "ordinary"
    );
    assert_eq!(module_export_name(&export.specifiers[1].exported), "�d800");
    let Statement::VariableDeclaration(declaration) = &parsed.program.body[1] else {
        panic!("expected variable declaration");
    };
    let Some(Expression::ObjectExpression(object)) = &declaration.declarations[0].init else {
        panic!("expected object");
    };
    let names: Vec<_> = object
        .properties
        .iter()
        .map(|property| {
            property_key_name(&property.as_property().unwrap().key)
                .unwrap()
                .into_owned()
        })
        .collect();
    assert_eq!(names, ["�d800", "42", "�d800�dc00", "�d800�dc00"]);
    for (statement, expected) in
        parsed.program.body[2..]
            .iter()
            .zip(["�d800�dc00", "�d800�dc00", "plain"])
    {
        let Statement::ExpressionStatement(statement) = statement else {
            panic!("expected expression")
        };
        let actual = match &statement.expression {
            Expression::ComputedMemberExpression(member) => computed_property_name(member),
            Expression::StaticMemberExpression(member) => {
                Some(Cow::Borrowed(member.property.name.as_str()))
            }
            _ => panic!("expected member"),
        };
        assert_eq!(actual.as_deref(), Some(expected));
    }
}

#[test]
fn legacy_surrogate_flag_distinguishes_literal_marker_text() {
    let source = fixture("flags.ts");
    let allocator = Allocator::default();
    let parsed = crate::ast::parse(
        std::path::Path::new("flags.ts"),
        &allocator,
        &source,
        SourceType::ts(),
    );
    assert!(parsed.diagnostics.is_empty());
    let Statement::VariableDeclaration(declaration) = &parsed.program.body[0] else {
        panic!("expected declaration")
    };
    let Some(Expression::ArrayExpression(array)) = &declaration.declarations[0].init else {
        panic!("expected array")
    };
    let flags: Vec<_> = array
        .elements
        .iter()
        .map(|element| {
            let Expression::StringLiteral(literal) = element.as_expression().unwrap() else {
                panic!("expected literal")
            };
            js_literal_has_lone_surrogate(literal)
        })
        .collect();
    assert_eq!(flags, [true, false, true, false, false]);
}
