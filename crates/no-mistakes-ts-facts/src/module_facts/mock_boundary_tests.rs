use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[test]
fn mock_boundary_syntax_requires_static_isolated_replacements() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/typescript-module-facts/mock-boundaries.ts");
    let source = std::fs::read_to_string(path).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let facts = extract_with_arguments(&parsed.program, false);
    let calls = &facts.module_mock_calls;
    let mut full: Vec<_> = calls
        .iter()
        .filter(|call| call.full)
        .map(|call| call.specifier.as_deref().unwrap())
        .collect();
    full.sort_unstable();
    assert_eq!(
        full,
        [
            "body-prefix",
            "construct",
            "function",
            "jest-alias",
            "jest-global",
            "values",
            "vi-global",
            "wrapped"
        ]
    );
    let mut uncertain: Vec<_> = calls
        .iter()
        .filter(|call| !call.full)
        .filter_map(|call| call.specifier.as_deref())
        .collect();
    uncertain.sort_unstable();
    assert_eq!(
        uncertain,
        [
            "actual",
            "argument-spread",
            "computed",
            "conditional",
            "deferred",
            "do-unmock",
            "dynamic-import",
            "empty",
            "eval",
            "extra-args",
            "factory-name",
            "flow",
            "function-call",
            "function-constructor",
            "generator",
            "getter",
            "method",
            "named-result",
            "named-value",
            "nested",
            "no-return",
            "object-spread",
            "optional-call",
            "optional-invocation",
            "optional-member",
            "params",
            "prototype",
            "rest",
            "returnless",
            "template-hole",
            "typeof-call",
            "unmock"
        ]
    );
    assert_eq!(
        calls.iter().filter(|call| call.specifier.is_none()).count(),
        2
    );
    for call in calls {
        assert_eq!(
            call.hoisted,
            !matches!(
                call.specifier.as_deref(),
                Some("deferred" | "unmock" | "do-unmock")
            )
        );
        if matches!(
            call.specifier.as_deref(),
            Some("function" | "values" | "generator")
        ) {
            assert!(call.factory_span.is_some(), "{call:?}");
        }
        if call.specifier.as_deref() == Some("factory-name") {
            assert!(call.factory_span.is_none());
            assert!(!call.automock);
        }
    }
    let actual_sources: Vec<_> = facts
        .module_mock_actual_spans
        .iter()
        .map(|span| &source[span.start as usize..span.end as usize])
        .collect();
    assert_eq!(
        actual_sources,
        [
            "Function('return 1')",
            "eval('1')",
            "v.importActual('./actual')"
        ]
    );
}
