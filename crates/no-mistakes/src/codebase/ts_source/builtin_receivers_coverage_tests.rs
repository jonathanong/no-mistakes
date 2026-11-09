use super::super::{collect, ProvenCalls};
use super::proven;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn source(name: &str) -> String {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    std::fs::read_to_string(root.join("src").join(name)).unwrap()
}

fn facts(text: &str) -> ProvenCalls {
    let allocator = Allocator::default();
    let program = Parser::new(&allocator, text, SourceType::mjs().with_typescript(true))
        .parse()
        .program;
    collect(&program)
}

#[test]
fn receiver_proofs_cover_callback_shapes_and_property_types() {
    let text = source("proof-coverage.mts");
    let found = proven(&text);
    for expected in [
        "n.toFixed",
        "n.toExponential",
        "text.trim",
        "flag.valueOf",
        "integer.toString",
        "input.target.toFixed",
        "input.label.trim",
        "unknown.map",
        "dateValue.toString",
        "promiseValue.then",
        "arrayValue.map",
        "Array.from",
    ] {
        assert!(
            found.iter().any(|call| call == expected),
            "{expected}: {found:?}"
        );
    }
    for excluded in [
        "input.computed.trim",
        "qualified.map",
        "literal.toString",
        "raw.run",
    ] {
        assert!(
            !found.iter().any(|call| call.contains(excluded)),
            "{excluded}: {found:?}"
        );
    }
    let proof = facts(&text);
    let unknown_map = text.find("unknown.map(").unwrap();
    assert!(proof.calls.contains(&(unknown_map as u32)));
    let unknown_callback = unknown_map + text[unknown_map..].find("value.toFixed(1)").unwrap();
    assert!(!proof.calls.contains(&(unknown_callback as u32)));
    let unsupported_constructor = text.find("(new Unknown()).map(").unwrap();
    assert!(!proof.calls.contains(&(unsupported_constructor as u32)));
    let nested_function = text.find("n.toFixed(5)").unwrap();
    let direct_after_nested = text.find("n.toFixed(6)").unwrap();
    assert!(!proof.calls.contains(&(nested_function as u32)));
    assert!(proof.calls.contains(&(direct_after_nested as u32)));
    let deep_receiver = text.find("date!!!!!!!!!!.toString()").unwrap();
    assert!(!proof.calls.contains(&(deep_receiver as u32)));
    let callbacks = proof.callbacks;
    assert!(callbacks.len() >= 6, "{callbacks:?}");
    assert!(callbacks
        .values()
        .any(|positions| *positions == Some(&[0][..])));
}
