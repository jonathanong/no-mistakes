use super::*;
use arguments::ArgumentValue;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::{json, Value};

fn tagged(value: &ArgumentValue) -> Value {
    match value {
        ArgumentValue::Unknown => json!({"kind":"unknown"}),
        ArgumentValue::Scalar(value) => json!({"kind":"scalar","value":value}),
        ArgumentValue::Array(items) => {
            json!({"kind":"array","items":items.iter().map(tagged).collect::<Vec<_>>()})
        }
        ArgumentValue::Object(properties) => {
            json!({"kind":"object","properties":properties.iter().map(|(k,v)|(k.clone(),tagged(v))).collect::<serde_json::Map<_,_>>()})
        }
    }
}

fn numeric_literals(value: &mut Value) {
    if value["kind"] == "scalar" && value["value"].is_number() {
        // JavaScript numeric literals and predicate equality use finite f64.
        value["value"] = json!(value["value"].as_f64().unwrap());
    }
    match value {
        Value::Array(items) => items.iter_mut().for_each(numeric_literals),
        Value::Object(properties) => properties.values_mut().for_each(numeric_literals),
        _ => {}
    }
}

#[test]
fn saved_projection_matrix_preserves_every_value_offset_and_spread_boundary() {
    let root = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates");
    for name in [
        "literal-matrix.mjs",
        "projection-budgets.mjs",
        "alias-negative.mjs",
        "alias-boundaries.mjs",
        "direct-eval.cjs",
        "shadowed-eval.cjs",
        "with-scope.cjs",
    ] {
        let source = std::fs::read_to_string(root.join(name)).unwrap();
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::from_path(name).unwrap()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{name}: {:?}",
            parsed.diagnostics
        );
        let semantic = SemanticBuilder::new()
            .with_build_nodes(true)
            .build(&parsed.program)
            .semantic;
        // Exercise the private projector's own defensive contract directly;
        // prepared semantic diagnostics are separately required to suppress proof.
        let projected = arguments::collect(&semantic, &TypeScriptModuleFacts::default());
        let mut calls: Vec<_> = projected.iter().map(|(offset, values)| json!({
            "offset":offset,"line":source[..*offset as usize].bytes().filter(|b|*b==b'\n').count()+1,
            "values":values.values.iter().map(tagged).collect::<Vec<_>>(),"spreadFrom":values.spread_from,
        })).collect();
        calls.sort_by_key(|call| call["offset"].as_u64().unwrap());
        let actual =
            json!({"sourceType":if name.ends_with(".cjs") {"script"}else{"module"},"calls":calls});
        let mut expected: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(format!("{name}.expected.json"))).unwrap(),
        )
        .unwrap();
        numeric_literals(&mut expected);
        assert_eq!(actual, expected, "{name}");
    }
}
