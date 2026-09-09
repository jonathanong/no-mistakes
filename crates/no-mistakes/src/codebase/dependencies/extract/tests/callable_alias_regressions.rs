use super::*;

#[test]
fn computed_object_keys_retain_object_symbol_ownership() {
    let source = r#"
        import { alpha as key } from "./source.mts";
        const obj = { [key]: 1 };
        export const api = obj;
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    assert!(facts.symbol_references.iter().any(|reference| {
        reference.caller.as_deref() == Some("obj") && reference.callee == "key"
    }));
}

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
    assert!(facts
        .callable_aliases
        .iter()
        .any(|alias| { alias.local == "api.alias" && alias.invalidated_at.is_some() }));
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

#[test]
fn callable_alias_invalidation_preserves_calls_before_the_assignment() {
    let source = "function target() {} const alias = target; alias(); alias = injected; alias();";
    let facts = facts(source);
    let calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.callee == "alias")
        .collect();

    assert_eq!(calls.len(), 2);
    assert_eq!(
        calls[0].target_identity,
        CallTargetIdentity::RepositoryFunction
    );
    assert_eq!(calls[1].target_identity, CallTargetIdentity::Unknown);
    assert!(facts.callable_aliases.iter().any(|alias| {
        alias.local == "alias"
            && alias.target == "target"
            && alias.invalidated_at == Some(source.find("alias = injected").unwrap() as u32)
    }));
}
