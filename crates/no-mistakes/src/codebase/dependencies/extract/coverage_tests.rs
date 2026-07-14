use super::*;
use oxc_parser::Parser;

#[test]
fn top_level_type_bindings_shadow_file_fallback_references() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "type Local = { value: string };\ntype Public = Local;",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    let refs: Vec<_> = facts
        .symbol_references
        .iter()
        .map(|reference| (reference.caller.as_deref(), reference.callee.as_str()))
        .collect();
    assert_eq!(refs, vec![(Some("Public"), "Local"), (None, "Local")]);
}

#[test]
fn nested_jsx_member_references_are_recorded() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import { UI } from './source.mts';\nexport const view = <UI.Form.Input />;",
        SourceType::tsx(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert!(facts
        .symbol_references
        .iter()
        .any(|reference| reference.callee == "UI.Form.Input"));
}

#[test]
fn nested_type_binding_can_resolve_to_known_function_scope() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "export function outer() {
           function Local() {}
           interface Local { value: string }
           type Uses = Local.Member;
         }",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert!(facts.symbol_references.iter().any(|reference| {
        reference.caller.as_deref() == Some("outer") && reference.callee == "outer/Local.Member"
    }));
}

#[test]
fn parenthesized_default_object_expression_records_members() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import { alpha } from './source.mts';\nexport default (({ method() { return alpha; } }));",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert!(facts.symbol_references.iter().any(|reference| {
        reference.caller.as_deref() == Some("default/method") && reference.callee == "alpha"
    }));
}
