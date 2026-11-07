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
