use super::*;

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
