use super::*;
use oxc_parser::Parser;

#[test]
fn fixture_binding_aware_calls_preserve_import_alias_namespace_local_and_unknown_identity() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/call-reachability/bindings/index.ts");
    let source = std::fs::read_to_string(&fixture).expect("binding fixture should exist");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program_with_source(&parsed.program, &source);
    let calls = &facts.call_reachability;

    let typescript_calls: Vec<_> = calls
        .iter()
        .filter(|call| matches!(
        &call.binding,
        CallBinding::Import { module, export } if module == "typescript" && export == "createProgram"
    ))
        .collect();
    assert_eq!(typescript_calls.len(), 16, "{calls:#?}");
    assert!(typescript_calls
        .iter()
        .any(|call| call.caller.as_deref() == Some("localAliases")));
    assert!(calls.iter().any(|call| matches!(
        &call.binding,
        CallBinding::Import { module, export } if module == "node:path" && export == "resolve"
    ) && call.invocation_kind == "member"));
    assert!(calls.iter().any(|call| matches!(
        &call.binding,
        CallBinding::Import { module, export }
            if module == "node:path" && export == "posix.resolve"
    ) && call.invocation_kind == "member"));
    assert!(calls.iter().any(|call| matches!(
        &call.binding,
        CallBinding::Local { scope } if scope == "local"
    )));
    assert!(calls.iter().any(|call| matches!(
        &call.binding,
        CallBinding::Global { name } if name == "globalThis.setTimeout"
    )));
    assert!(calls.iter().any(|call| {
        matches!(
            &call.binding,
            CallBinding::Global { name } if name == "globalThis.URL"
        ) && call.invocation_kind == "construct"
    }));
    assert!(calls
        .iter()
        .any(|call| matches!(call.binding, CallBinding::Unresolved { .. })));
    assert!(calls.iter().any(|call| {
        call.caller.as_deref() == Some("shadowed")
            && matches!(
                &call.binding,
                CallBinding::Shadowed { name } if name == "makeProgram"
            )
    }));
    assert!(!calls.iter().any(|call| {
        call.caller.as_deref() == Some("shadowedRequire")
            && matches!(
                &call.binding,
                CallBinding::Import { module, .. } if module == "typescript"
            )
    }));
    assert!(calls.iter().any(|call| {
        call.caller.as_deref() == Some("shadowedAlias")
            && matches!(
                &call.binding,
                CallBinding::Local { scope } if scope == "shadowedAlias/first"
            )
    }));
    assert!(
        calls.iter().any(|call| matches!(
            &call.binding,
            CallBinding::Import { module, export }
                if module == "typescript" && export == "createProgram"
        ) && call.caller.as_deref() == Some("run")),
        "{calls:#?}"
    );
    assert!(
        calls.iter().any(|call| matches!(
            &call.binding,
            CallBinding::Import { module, export }
                if module == "tooling" && export == "tooling.createProgram"
        ) && call.caller.as_deref() == Some("run")),
        "{calls:#?}"
    );
    assert!(calls.iter().any(|call| {
        call.caller.is_none()
            && matches!(&call.binding, CallBinding::Local { scope } if scope == "hoistedTopLevel")
    }), "{calls:#?}");
    assert!(
        calls.iter().any(|call| matches!(
            &call.binding,
            CallBinding::Import { module, export }
                if module == "./late" && export == "lateImport"
        ) && call.caller.is_none()),
        "late import should be hoisted: {calls:#?}"
    );
    assert!(
        calls.iter().any(|outer| {
            let CallBinding::Local { scope } = &outer.binding else {
                return false;
            };
            outer.caller.as_deref() == Some("run")
                && scope.starts_with("run/<anonymous:")
                && calls.iter().any(|inner| {
                    inner.caller.as_deref() == Some(scope)
                        && matches!(
                            &inner.binding,
                            CallBinding::Import { module, export }
                                if module == "typescript" && export == "createProgram"
                        )
                })
        }),
        "IIFE body should be reachable from run: {calls:#?}"
    );
    assert!(
        calls.iter().any(|call| {
            call.caller.as_deref() == Some("constructLocal")
                && matches!(&call.binding, CallBinding::Local { scope } if scope == "LocalClass")
                && call.invocation_kind == "construct"
        }),
        "constructor call should resolve to its class: {calls:#?}"
    );
    assert!(calls.iter().any(|call| {
        call.caller.as_deref() == Some("LocalClass")
            && matches!(&call.binding, CallBinding::Local { scope } if scope == "LocalClass/constructor")
    }), "class should reach its constructor body: {calls:#?}");
    for name in ["topLevelValue", "blockValue"] {
        assert!(calls.iter().any(|call| {
            call.caller.is_none()
                && matches!(&call.binding, CallBinding::Shadowed { name: binding } if binding == name)
        }), "{name} should not be a global: {calls:#?}");
    }
    for caller in ["parameterStopsOuterCallable", "localStopsOuterCallable"] {
        assert!(calls.iter().any(|call| {
            call.caller.as_deref() == Some(caller)
                && matches!(&call.binding, CallBinding::Shadowed { name } if name == "outerCallable")
        }), "{caller} should stop before the outer callable: {calls:#?}");
    }
    let lexical_tdz_calls: Vec<_> = calls
        .iter()
        .filter(|call| call.caller.as_deref() == Some("lexicalTdz") && call.callee == "makeProgram")
        .collect();
    assert_eq!(lexical_tdz_calls.len(), 2, "{calls:#?}");
    assert!(matches!(
        lexical_tdz_calls[0].binding,
        CallBinding::Shadowed { .. }
    ));
    assert!(matches!(
        lexical_tdz_calls[1].binding,
        CallBinding::Local { .. }
    ));
}

#[test]
fn fixture_binding_aware_call_retains_reexport_module_and_export_identity() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/call-reachability/reexports/caller.ts");
    let source = std::fs::read_to_string(&fixture).expect("re-export fixture should exist");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, &source);

    assert!(facts.call_reachability.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && matches!(
                &call.binding,
                CallBinding::Import { module, export }
                    if module == "./barrel" && export == "createProgram"
            )
    }));
}
