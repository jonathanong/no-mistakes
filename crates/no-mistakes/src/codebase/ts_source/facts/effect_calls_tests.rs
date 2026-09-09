use super::effect_calls::*;
use crate::codebase::dependencies::extract::{
    extract_import_facts_from_program_with_source, CallTargetIdentity, FunctionCall, InvocationKind,
};
use oxc_allocator::Allocator;
use oxc_span::SourceType;
use std::path::Path;

#[test]
fn preserves_terminal_effects_on_non_identifier_receivers() {
    let source = "this.invalidate(); factory().invalidate();";
    let allocator = Allocator::default();
    let parsed = crate::ast::parse(
        Path::new("non-identifier-effects.ts"),
        &allocator,
        source,
        SourceType::ts(),
    );
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    let effects = collect_effect_calls(
        &facts.function_calls,
        &EffectNames::from([("invalidate".to_string(), Some("cache".to_string()))]),
    );

    assert_eq!(effects.len(), 2);
    assert!(effects.iter().all(|effect| effect.callee == "invalidate"));
    assert!(effects
        .iter()
        .all(|effect| effect.category.as_deref() == Some("cache")));
    assert_eq!(facts.unknown_calls.len(), 1);
    assert_eq!(facts.unknown_calls[0].line, 1);
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.callee == "<unknown>.invalidate"));
}

#[test]
fn deduplicates_scoped_copies_by_offset_and_keeps_the_canonical_owner() {
    let call = |offset, caller: Option<&str>, syntactic_caller: &str| FunctionCall {
        caller: caller.map(str::to_owned),
        caller_id: None,
        syntactic_caller: Some(syntactic_caller.to_string()),
        callee: "flush".to_string(),
        line: 1,
        offset,
        is_callback: false,
        invocation: InvocationKind::Call,
        target_identity: CallTargetIdentity::Unknown,
        callee_binding_scope: None,
        static_arg: None,
        static_cwd: None,
    };
    let effects = collect_effect_calls(
        &[
            call(10, Some("run/<anonymous:1>"), "nested"),
            call(10, Some("run"), "run"),
            call(30, Some("run"), "run"),
        ],
        &EffectNames::from([("flush".to_string(), None)]),
    );

    assert_eq!(effects.len(), 2, "{effects:#?}");
    assert!(effects
        .iter()
        .all(|effect| effect.caller.as_deref() == Some("run")));
}

#[test]
fn keeps_nested_effects_that_share_an_ast_start_offset() {
    let source = "function run() { start().stop(); }";
    let allocator = Allocator::default();
    let parsed = crate::ast::parse(
        Path::new("nested-effects.ts"),
        &allocator,
        source,
        SourceType::ts(),
    );
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    let effects = collect_effect_calls(
        &facts.function_calls,
        &EffectNames::from([("start".to_string(), None), ("stop".to_string(), None)]),
    );

    assert_eq!(effects.len(), 2, "{effects:#?}");
    let mut callees = effects
        .iter()
        .map(|effect| effect.callee.as_str())
        .collect::<Vec<_>>();
    callees.sort_unstable();
    assert_eq!(callees, ["start", "stop"]);
}

#[test]
fn excludes_synthetic_callback_construction_transitions() {
    let call = |is_callback| FunctionCall {
        caller: Some("Derived".to_string()),
        caller_id: None,
        syntactic_caller: Some("Derived".to_string()),
        callee: "Base".to_string(),
        line: 1,
        offset: u32::from(is_callback),
        is_callback,
        invocation: InvocationKind::Construct,
        target_identity: CallTargetIdentity::RepositoryFunction,
        callee_binding_scope: None,
        static_arg: None,
        static_cwd: None,
    };
    let effects = collect_effect_calls(
        &[call(true), call(false)],
        &EffectNames::from([("Base".to_string(), None)]),
    );

    assert_eq!(effects.len(), 1, "{effects:#?}");
    assert_eq!(effects[0].callee, "Base");
}
