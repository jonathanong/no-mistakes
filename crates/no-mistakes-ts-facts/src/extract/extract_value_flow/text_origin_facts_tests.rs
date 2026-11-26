use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::sync::Arc;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/codebase-analysis/text-origin-facts/fixture/shapes.ts"
));

#[test]
fn unsupported_syntax_cannot_reuse_a_widened_object_literal_origin() {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, SOURCE, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let value = ValueExpression::Object(
        Arc::new(
            [("safe".to_string(), ValueExpression::Unknown)]
                .into_iter()
                .collect(),
        ),
        false,
    );
    let mut checked = Vec::new();
    for statement in &parsed.program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declaration in &declaration.declarations {
            let BindingPattern::BindingIdentifier(name) = &declaration.id else {
                continue;
            };
            let Some(initializer) = &declaration.init else {
                continue;
            };
            let name = name.name.as_str();
            if name == "mismatchedParts" {
                assert!(
                    origin(initializer, &ValueExpression::Derived(Vec::new().into())).is_none()
                );
            } else if name == "plain" {
                let origin = origin(initializer, &value).unwrap();
                assert!(origin.fields.get("safe").unwrap().literal);
            } else {
                assert!(origin(initializer, &value).is_none(), "{name}");
            }
            checked.push(name);
        }
    }
    assert_eq!(
        checked,
        [
            "plain",
            "spread",
            "computed",
            "bigint",
            "accessor",
            "method",
            "mismatchedParts"
        ]
    );
}

#[test]
fn assignment_pattern_parameter_defaults_retain_the_opaque_initializer() {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, SOURCE, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        SOURCE,
        false,
        false,
        true,
    );
    let mut collector = ImportCollector {
        value_flow: facts.value_flow,
        ..Default::default()
    };
    collector.value_flow.parameters.truncate(1);
    assert_eq!(collector.value_flow.parameters[0].name, "value");
    let mut program = parsed.program;
    let function = program
        .body
        .iter_mut()
        .find_map(|statement| match statement {
            Statement::FunctionDeclaration(function) => Some(function),
            _ => None,
        })
        .unwrap();
    let (first, later) = function.params.items.split_first_mut().unwrap();
    let BindingPattern::ObjectPattern(pattern) = &mut first.pattern else {
        panic!("object parameter");
    };
    std::mem::swap(&mut later[0].pattern, &mut pattern.properties[0].value);
    collector.note_text_parameter_default(0, &later[0], &ValueExpression::Unknown);
    assert!(collector.opaque_text_parameter_defaults.contains(&0));
}

#[test]
fn identifier_projection_keeps_only_scalar_bindings_and_first_composed_arguments() {
    let literal = TextOrigin {
        literal: true,
        ..Default::default()
    };
    let object = TextOrigin {
        fields: [("field".into(), literal.clone())].into_iter().collect(),
        ..Default::default()
    };
    let composed = TextOrigin {
        parts: vec![None, Some(literal.clone()), Some(object.clone())],
        ..Default::default()
    };
    let origins = TextOrigins {
        arguments: [
            ((1, 2), vec![Some(composed), Some(literal.clone())]),
            ((3, 4), vec![Some(literal.clone())]),
            ((5, 6), vec![Some(object.clone())]),
            ((7, 8), vec![None]),
            ((9, 10), vec![]),
        ]
        .into_iter()
        .collect(),
        bindings: [(
            0,
            [
                ("literal".into(), [(0, literal)].into_iter().collect()),
                ("object".into(), [(0, object)].into_iter().collect()),
            ]
            .into_iter()
            .collect(),
        )]
        .into_iter()
        .collect(),
    };
    let selected = origins.identifier_scalars().unwrap();
    assert_eq!(selected.arguments.len(), 1);
    assert_eq!(selected.arguments[&(1, 2)].len(), 1);
    let first = selected.argument((1, 2), 0).unwrap();
    assert_eq!(first.parts.len(), 3);
    assert!(first.parts[0].is_none());
    assert!(first.parts[1].as_ref().unwrap().literal);
    assert!(first.parts[2].is_none());
    assert!(selected.binding(0, "literal", 0).unwrap().literal);
    assert!(selected.binding(0, "object", 0).is_none());
    assert!(TextOrigins::default().identifier_scalars().is_none());
}
