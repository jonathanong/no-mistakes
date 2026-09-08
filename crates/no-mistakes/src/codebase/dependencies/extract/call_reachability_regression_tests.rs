use super::*;
use oxc_parser::Parser;

fn binding_fixture_call_facts() -> ImportFacts {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/call-reachability/bindings/index.ts");
    let source = std::fs::read_to_string(&fixture).expect("binding fixture should exist");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, &source)
}

#[test]
fn function_bodies_predeclare_lexical_and_var_bindings_for_call_identity() {
    let facts = binding_fixture_call_facts();
    for caller in [
        "functionBodyTdz",
        "arrowBodyTdz",
        "functionExpressionBodyTdz",
    ] {
        let first = facts
            .call_reachability
            .iter()
            .find(|call| call.caller.as_deref() == Some(caller) && call.callee == "makeProgram")
            .unwrap_or_else(|| panic!("missing call for {caller}: {:#?}", facts.call_reachability));
        assert!(
            matches!(first.binding, CallBinding::Shadowed { .. }),
            "{caller}: {first:#?}"
        );
    }
}

#[test]
fn exported_object_members_and_program_blocks_retain_local_call_edges() {
    let facts = binding_fixture_call_facts();
    assert!(facts.call_reachability.iter().any(|call| {
        call.caller.as_deref() == Some("exportedApi/run")
            && matches!(
                &call.binding,
                CallBinding::Import { module, export }
                    if module == "typescript" && export == "createProgram"
            )
    }));
    assert!(facts.call_reachability.iter().any(|call| {
        call.caller.is_none()
            && matches!(&call.binding, CallBinding::Local { scope } if scope == "blockCallable")
    }));
}

#[test]
fn local_classes_are_callable_and_loop_bindings_do_not_leak() {
    let facts = binding_fixture_call_facts();
    assert!(facts.call_reachability.iter().any(|call| {
        call.caller.as_deref() == Some("localClassConstructor")
            && matches!(
                &call.binding,
                CallBinding::Local { scope } if scope == "localClassConstructor/Worker"
            )
            && call.invocation_kind == "construct"
    }));

    let post_scope_imports = facts
        .call_reachability
        .iter()
        .filter(|call| {
            call.caller.as_deref() == Some("loopBindingsDoNotLeak")
                && matches!(
                    &call.binding,
                    CallBinding::Import { module, export }
                        if module == "typescript" && export == "createProgram"
                )
        })
        .count();
    assert_eq!(post_scope_imports, 4, "{:#?}", facts.call_reachability);
}
