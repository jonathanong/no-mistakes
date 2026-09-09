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

#[test]
fn immutable_object_aliases_preserve_callable_member_identity() {
    let facts = facts(
        "function target() {} const api = { run: target }; const facade = api; facade.run();",
    );

    assert!(facts.callable_aliases.iter().any(|alias| {
        alias.local == "facade.run" && alias.target == "api.run" && alias.invalidated_at.is_none()
    }));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "facade.run"
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}

#[test]
fn chained_immutable_object_aliases_preserve_callable_members() {
    let facts = facts(
        "function target() {} const api = { run: target }; const facade = api; const secondFacade = facade; secondFacade.run();",
    );

    assert!(facts
        .callable_aliases
        .iter()
        .any(|alias| { alias.local == "secondFacade.run" && alias.target == "facade.run" }));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "secondFacade.run"
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}

#[test]
fn class_aliases_only_materialize_static_callable_members() {
    let facts = facts(
        "class Service { static run() {} instance() {} } const Alias = Service; Alias.run(); Alias.instance();",
    );

    assert!(facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "Alias.run"));
    assert!(!facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "Alias.instance"));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "Alias.run" && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "Alias.instance" && call.target_identity == CallTargetIdentity::Unknown
    }));
}

#[test]
fn class_expression_aliases_only_materialize_static_callable_members() {
    let facts = facts(
        "const Service = class { static run() {} instance() {} }; const Alias = Service; Alias.run(); Alias.instance();",
    );

    assert!(facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "Alias.run"));
    assert!(!facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "Alias.instance"));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "Alias.run" && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "Alias.instance" && call.target_identity == CallTargetIdentity::Unknown
    }));
}

#[test]
fn materialized_aliases_keep_their_declaring_callable_owner() {
    let facts = facts(
        "function target() {} function outer() { const api = { run: target }; const facade = api; facade.run(); }",
    );
    let outer_id = facts
        .callable_scope_ids
        .iter()
        .find_map(|(id, scope)| (scope == "outer").then_some(*id))
        .expect("outer callable identity");

    assert!(facts.callable_aliases.iter().any(|alias| {
        alias.local == "facade.run"
            && alias.scope.as_deref() == Some("outer")
            && alias.scope_id == Some(outer_id)
    }));
}

#[test]
fn increment_invalidates_a_local_callable_alias() {
    let source = "function target() {} const alias = target; alias++; alias();";
    let facts = facts(source);

    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "alias" && call.target_identity == CallTargetIdentity::Unknown
    }));
    assert!(facts.callable_aliases.iter().any(|alias| {
        alias.local == "alias"
            && alias.target == "target"
            && alias.invalidated_at == Some(source.find("alias++").unwrap() as u32)
    }));
}

#[test]
fn duplicate_object_keys_keep_the_last_callable_write() {
    let facts = facts(
        "function target() {} function later() {} const calls = { run: target, run: 0, run: later }; calls.run();",
    );

    assert!(facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "calls.run" && alias.target == "later"));
    assert!(!facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "calls.run" && alias.target == "target"));
}

#[test]
fn non_callable_duplicate_key_drops_the_member_alias() {
    let facts = facts("function target() {} const calls = { run: target, run: 0 }; calls.run();");

    assert!(!facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "calls.run"));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "calls.run" && call.target_identity == CallTargetIdentity::Unknown
    }));
}

#[test]
fn later_object_spread_drops_earlier_member_aliases() {
    let facts = facts(
        "function target() {} const replacement = {}; const calls = { run: target, ...replacement }; calls.run();",
    );

    assert!(!facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "calls.run"));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "calls.run" && call.target_identity == CallTargetIdentity::Unknown
    }));
}
