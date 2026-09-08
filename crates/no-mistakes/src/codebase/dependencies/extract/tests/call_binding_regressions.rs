use super::*;

#[test]
fn fixture_object_function_properties_track_static_scopes() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/object-function-property.mts",
    );
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);
    let scopes: Vec<_> = facts
        .imports
        .iter()
        .map(|import| import.function_scope.as_deref())
        .collect();

    assert_eq!(scopes, vec![Some("loaders/load"), Some("loaders/fallback")]);
}

#[test]
fn fixture_object_arrow_properties_track_static_scopes() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/object-function-arrow-property.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(
        facts.imports[0].function_scope.as_deref(),
        Some("loaders/lazy")
    );
}

#[test]
fn fixture_computed_function_keys_are_visited_under_parent_scope() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/computed-function-keys.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);
    let imports: Vec<_> = facts
        .imports
        .iter()
        .map(|import| (import.specifier.as_str(), import.function_scope.as_deref()))
        .collect();

    assert_eq!(
        imports,
        vec![
            ("./key.mts", Some("loaders")),
            ("./loaded.mts", Some("loaders")),
            ("./method-key.mts", None),
            ("./loaded.mts", Some("Loader"))
        ]
    );
}

#[test]
fn fixture_anonymous_function_expression_records_anonymous_scope() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/anonymous-function-expression.mts",
    );
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(
        facts.imports[0].function_scope.as_deref(),
        Some("<anonymous:1>")
    );
}

#[test]
fn function_call_facts_preserve_source_line_and_callback_provenance() {
    let source = "function run() {\n  helper();\n  (() => helper())();\n}";
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    let direct = facts
        .function_calls
        .iter()
        .find(|call| call.callee == "helper" && call.caller.as_deref() == Some("run"))
        .expect("direct helper call");
    assert_eq!(direct.line, 2);
    assert!(!direct.is_callback);
    assert_eq!(direct.invocation, InvocationKind::Call);
    assert!(facts.function_calls.iter().any(|call| {
        call.is_callback
            && call.invocation == InvocationKind::Callback
            && call.caller.as_deref() == Some("run")
            && call.line == 0
    }));
}

#[test]
fn static_computed_member_calls_keep_a_resolvable_callee() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/computed-member-calls.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program_with_source(&parsed.program, &source);
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "page.waitForTimeout" && call.target_identity == CallTargetIdentity::Unknown
    }));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "playwright.test" && call.target_identity == CallTargetIdentity::ModuleExport
    }));
    assert!(facts.unknown_calls.is_empty());
}

#[test]
fn call_facts_preserve_runtime_import_aliases_and_named_reexports() {
    let source = r#"
        import { source as alias } from "./source.mts";
        export { alias as publicAlias };
        export { publicAlias as forwarded } from "./barrel.mts";
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    assert!(facts.imported_bindings.iter().any(|binding| {
        binding.specifier == "./source.mts"
            && binding.local == "alias"
            && binding.imported == "source"
            && !binding.is_type_only
    }));
    assert!(facts.exported_bindings.iter().any(|binding| {
        binding.specifier.is_none() && binding.local == "alias" && binding.exported == "publicAlias"
    }));
    assert!(facts.exported_bindings.iter().any(|binding| {
        binding.specifier.as_deref() == Some("./barrel.mts")
            && binding.local == "publicAlias"
            && binding.exported == "forwarded"
    }));
}

#[test]
fn function_call_facts_distinguish_calls_from_construction() {
    let source = "function Widget() {}\nWidget();\nnew Widget();";
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "Widget" && call.line == 2 && call.invocation == InvocationKind::Call
    }));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "Widget" && call.line == 3 && call.invocation == InvocationKind::Construct
    }));
}

#[test]
fn tagged_template_facts_use_named_call_binding_classification() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/tagged-template-calls.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, &source);

    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "localTag"
            && call.caller.as_deref() == Some("taggedTemplates")
            && call.invocation == InvocationKind::Call
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "importedTag"
            && call.caller.as_deref() == Some("taggedTemplates")
            && call.invocation == InvocationKind::Call
            && call.target_identity == CallTargetIdentity::ModuleExport
    }));
    assert_eq!(
        facts
            .function_calls
            .iter()
            .filter(|call| matches!(call.callee.as_str(), "localTag" | "importedTag"))
            .count(),
        2,
        "computed and dynamically produced tags must not be guessed as named calls"
    );
}

#[test]
fn call_facts_classify_only_unshadowed_global_bindings_as_global() {
    let source = r#"
        setTimeout(() => {}, 1);
        globalThis.setTimeout(() => {}, 1);
        function local() {
          const setTimeout = () => {};
          const globalThis = { setTimeout() {} };
          setTimeout();
          globalThis.setTimeout();
        }
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    assert_eq!(
        facts
            .function_calls
            .iter()
            .filter(|call| call.target_identity == CallTargetIdentity::Global)
            .map(|call| call.callee.as_str())
            .collect::<Vec<_>>(),
        vec!["setTimeout", "globalThis.setTimeout"]
    );
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "setTimeout"
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "globalThis.setTimeout"
            && call.line > 4
            && call.target_identity == CallTargetIdentity::Unknown
    }));
}

#[test]
fn call_facts_never_classify_later_lexical_bindings_as_globals() {
    let source = r#"
        setTimeout();
        globalThis.setTimeout();
        function run() {
          setTimeout();
          globalThis.setTimeout();
          const setTimeout = () => {};
          const globalThis = { setTimeout() {} };
        }
        const setTimeout = () => {};
        const globalThis = { setTimeout() {} };
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    assert!(!facts.function_calls.iter().any(|call| {
        matches!(call.callee.as_str(), "setTimeout" | "globalThis.setTimeout")
            && call.target_identity == CallTargetIdentity::Global
    }));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "globalThis.setTimeout"
            && call.target_identity == CallTargetIdentity::Unknown
    }));
}

#[test]
fn call_facts_keep_module_block_bindings_inside_their_block() {
    let source = r#"
        {
          const setTimeout = () => {};
          setTimeout();
        }
        setTimeout();
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    let calls = facts
        .function_calls
        .iter()
        .filter(|call| call.callee == "setTimeout")
        .collect::<Vec<_>>();

    assert_eq!(calls.len(), 2);
    assert_eq!(
        calls[0].target_identity,
        CallTargetIdentity::RepositoryFunction
    );
    assert_eq!(calls[1].target_identity, CallTargetIdentity::Global);
}

#[test]
fn callable_alias_invalidation_tracks_destructuring_targets_and_lexical_bindings() {
    let source = r#"
        function target() {}
        function replacement() {}
        function outer() {
          const alias = target;
          function inner() {
            const alias = replacement;
            ({ alias } = source);
          }
          alias();
        }
        const moduleAlias = target;
        ({ moduleAlias } = source);
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    assert!(facts.callable_aliases.iter().any(|alias| {
        alias.scope.as_deref() == Some("outer")
            && alias.local == "alias"
            && alias.target == "target"
    }));
    assert!(!facts
        .callable_aliases
        .iter()
        .any(|alias| { alias.scope.as_deref() == Some("outer/inner") && alias.local == "alias" }));
    assert!(
        !facts
            .callable_aliases
            .iter()
            .any(|alias| alias.scope.is_none() && alias.local == "moduleAlias"),
        "object destructuring assignment must invalidate a module alias"
    );
}

#[test]
fn callable_alias_invalidation_handles_object_property_defaults_and_ignores_member_writes() {
    let source = r#"
        function target() {}
        const shorthand = target;
        const renamed = target;
        const fallback = target;
        const rest = target;
        const retained = target;

        ({ shorthand, original: renamed, fallback = replacement, ...rest } = source);
        holder.retained = replacement;
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    for invalidated in ["shorthand", "renamed", "fallback", "rest"] {
        assert!(
            !facts
                .callable_aliases
                .iter()
                .any(|alias| alias.scope.is_none() && alias.local == invalidated),
            "object assignment must invalidate {invalidated}"
        );
    }
    assert!(facts
        .callable_aliases
        .iter()
        .any(|alias| alias.scope.is_none()
            && alias.local == "retained"
            && alias.target == "target"));
}

#[test]
fn call_facts_record_unknown_calls_with_location_and_kind() {
    let source = "(condition ? left : right)();\nnew (condition ? Left : Right)();";
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    assert_eq!(
        facts.unknown_calls,
        vec![
            UnknownCall {
                caller: None,
                caller_id: None,
                line: 1,
                offset: 0,
                invocation: InvocationKind::Call,
            },
            UnknownCall {
                caller: None,
                caller_id: None,
                line: 2,
                offset: source.find("new ").unwrap() as u32,
                invocation: InvocationKind::Construct,
            },
        ]
    );
}

#[test]
fn call_facts_keep_namespace_reexports_out_of_transparent_star_sources() {
    let source = r#"
        export * from "./transparent.mts";
        export * as namespace from "./namespace.mts";
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    assert_eq!(facts.star_reexport_specifiers, vec!["./transparent.mts"]);
    assert!(facts.exported_bindings.iter().any(|binding| {
        binding.specifier.as_deref() == Some("./namespace.mts")
            && binding.local == "*"
            && binding.exported == "namespace"
    }));
}

#[test]
fn sequence_callees_use_only_a_simple_final_operand() {
    let source = "function helper() {} (0, helper)(); (0, factory())();";
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "helper" && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
    assert!(facts.unknown_calls.iter().any(|call| call.offset > 0));
}

#[test]
fn call_facts_predeclare_later_function_declarations_as_callable() {
    let source = r#"
        run();
        function run() {}
        function outer() {
          later();
          function later() {}
        }
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    for (caller, callee, scope) in [
        (None, "run", "run"),
        (Some("outer"), "later", "outer/later"),
    ] {
        assert!(facts.function_calls.iter().any(|call| {
            call.caller.as_deref() == caller
                && call.callee == callee
                && call.target_identity == CallTargetIdentity::RepositoryFunction
        }));
        assert!(facts
            .callable_scopes
            .iter()
            .any(|candidate| candidate == scope));
    }
}

#[test]
fn call_facts_hoist_var_bindings_out_of_module_and_function_blocks() {
    let source = r#"
        setTimeout();
        { var setTimeout = () => {}; }
        function run() {
          setInterval();
          { var setInterval = () => {}; }
        }
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    assert!(!facts.function_calls.iter().any(|call| {
        matches!(call.callee.as_str(), "setTimeout" | "setInterval")
            && call.target_identity == CallTargetIdentity::Global
    }));
}

#[test]
fn call_facts_do_not_hoist_class_static_block_vars_into_the_module() {
    let source = r#"
        setTimeout();
        class Timers {
          static { var setTimeout = () => {}; }
        }
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "setTimeout" && call.target_identity == CallTargetIdentity::Global
    }));
}

#[test]
fn call_facts_scope_loop_and_switch_lexical_bindings() {
    let source = r#"
        for (const setTimeout of callbacks) {
          setTimeout();
        }
        switch (mode) {
          case "before":
            setInterval();
            break;
          case "declares":
            const setInterval = () => {};
            setInterval();
            break;
        }
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    assert!(!facts.function_calls.iter().any(|call| {
        matches!(call.callee.as_str(), "setTimeout" | "setInterval")
            && call.target_identity == CallTargetIdentity::Global
    }));
}

// ── is_indexable / is_tsx_file ──────────────────────────────────────

#[test]
fn is_indexable_ts() {
    assert!(is_indexable(Path::new("a.ts")));
    assert!(is_indexable(Path::new("a.mts")));
    assert!(is_indexable(Path::new("a.tsx")));
    assert!(is_indexable(Path::new("a.cts")));
    assert!(is_indexable(Path::new("a.js")));
    assert!(is_indexable(Path::new("a.mjs")));
    assert!(is_indexable(Path::new("a.jsx")));
    assert!(is_indexable(Path::new("a.cjs")));
}

#[test]
fn is_indexable_rejects_non_ts() {
    assert!(!is_indexable(Path::new("a.rs")));
    assert!(!is_indexable(Path::new("a.json")));
    assert!(!is_indexable(Path::new("Makefile")));
}

#[test]
fn is_tsx_file_detects_tsx() {
    assert!(is_tsx_file(Path::new("a.tsx")));
    assert!(is_tsx_file(Path::new("a.jsx")));
    assert!(!is_tsx_file(Path::new("a.ts")));
    assert!(!is_tsx_file(Path::new("a.mts")));
}
