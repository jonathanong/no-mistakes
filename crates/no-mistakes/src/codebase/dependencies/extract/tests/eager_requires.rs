use super::*;

#[test]
fn do_while_tests_are_eager_only_when_the_body_reaches_them() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-runner-executions/conditional-requires/do-while-facts.mts"));
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    assert_eq!(
        facts.eager_require_specifiers,
        [
            "node:assert",
            "node:buffer",
            "node:constants",
            "node:crypto",
            "node:dns",
            "node:domain",
            "node:events"
        ]
    );
    assert_eq!(
        facts
            .imports
            .iter()
            .filter(|import| import.kind == ImportKind::Require)
            .count(),
        15
    );
}

#[test]
fn require_execution_facts_keep_edges_but_exclude_guarded_helpers() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-runner-executions/conditional-requires/execution-facts.mts"));
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    assert_eq!(
        facts.eager_require_specifiers,
        [
            "node:assert",
            "node:async_hooks",
            "node:buffer",
            "node:constants",
            "node:crypto",
            "node:dns",
            "node:domain",
            "node:events",
            "node:fs",
            "node:http",
            "node:https",
            "node:module",
            "node:net",
            // The first switch case test evaluates even when its body is skipped.
            "node:os",
            "node:path",
            "node:process",
            "node:punycode",
            "node:querystring",
            "node:readline",
            "node:stream",
            "node:tls",
            "node:tty",
            "node:url",
            "node:util",
            "node:vm",
            "node:worker_threads",
            "node:zlib",
        ]
    );
    // Guarded calls still belong to the canonical syntactic import graph.
    assert!(facts
        .imports
        .iter()
        .any(|import| import.kind == ImportKind::Require
            && import.specifier == "node:os"
            && import.function_scope.is_none()));
    assert!(facts
        .imports
        .iter()
        .any(|import| import.kind == ImportKind::Require
            && import.specifier == "node:os"
            && import.function_scope.is_some()));
    assert!(facts
        .imports
        .iter()
        .any(|import| import.kind == ImportKind::RequireResolve));
    assert!(facts.imports.iter().any(|import| import.computed));
}

#[test]
fn switch_case_tests_and_bigints_keep_only_proven_eager_requires() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-runner-executions/eager-switch-bigint",
    );
    for (file, eager) in [
        ("tests/switch-first.test.cjs", true),
        ("tests/switch-leading-default.test.cjs", true),
        ("tests/switch-later.test.cjs", false),
        ("tests/bigint-truthy.test.cjs", true),
        ("tests/bigint-zero.test.cjs", false),
        ("tests/bigint-if-true.test.cjs", true),
        ("tests/bigint-if-zero.test.cjs", false),
    ] {
        let source = std::fs::read_to_string(root.join(file)).unwrap();
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::from_path(file).unwrap()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{file}: {:?}",
            parsed.diagnostics
        );
        let facts = extract_import_facts_from_program_with_source(&parsed.program, &source);
        assert_eq!(
            facts
                .eager_require_specifiers
                .contains(&"../helpers/mock.cjs".to_string()),
            eager,
            "{file}"
        );
        assert!(
            facts
                .imports
                .iter()
                .any(|import| import.specifier == "../helpers/mock.cjs"),
            "{file}"
        );
    }
}
