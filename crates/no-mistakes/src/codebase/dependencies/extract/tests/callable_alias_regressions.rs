use super::*;

fn facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

#[test]
fn nested_member_reassignment_invalidates_the_object_literal_callable_alias() {
    let facts = facts(
        "function target() {} const api = { run() {}, alias: target }; function replace() { api.run = injected; api.alias = injected; } function call() { api.run(); api.alias(); }",
    );

    for member in ["api.run", "api.alias"] {
        assert!(facts.function_calls.iter().any(
            |call| call.callee == member && call.target_identity == CallTargetIdentity::Unknown
        ));
    }
    assert!(!facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "api.alias"));
}

#[test]
fn object_and_array_destructuring_preserve_immutable_callable_aliases() {
    let facts = facts(
        "function objectTarget() {} function arrayTarget() {} const { run: objectAlias } = { run: objectTarget }; const [arrayAlias] = [arrayTarget]; objectAlias(); arrayAlias();",
    );

    for (local, target) in [
        ("objectAlias", "objectTarget"),
        ("arrayAlias", "arrayTarget"),
    ] {
        assert!(facts
            .callable_aliases
            .iter()
            .any(|alias| alias.local == local && alias.target == target));
        assert!(facts.function_calls.iter().any(|call| {
            call.callee == local && call.target_identity == CallTargetIdentity::RepositoryFunction
        }));
    }
}
