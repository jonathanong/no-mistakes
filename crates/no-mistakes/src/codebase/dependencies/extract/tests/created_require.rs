use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;

const LOADS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/created-require/loads.mts"
));
const DYNAMIC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/created-require/dynamic.mts"
));
const FACTORIES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/created-require/factories.mts"
));
const NON_FACTORIES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/created-require/non-factories.mts"
));
const SHADOWED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/created-require/shadowed.mts"
));
const GLOBAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/created-require/global.cjs"
));

fn facts_of(source: &str, source_type: SourceType) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "parse errors: {:#?}",
        parsed.diagnostics
    );
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

fn require_literals(source: &str) -> Vec<(u32, String)> {
    let bytes = source.as_bytes();
    let mut found = Vec::new();
    let mut index = 0usize;
    while let Some(at) = source[index..].find("require(") {
        let start = index + at;
        let boundary = start == 0
            || !matches!(bytes[start - 1], b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.');
        index = start + "require(".len();
        if !boundary {
            continue;
        }
        let after = &source[index..];
        let Some(quote) = after.chars().next() else {
            continue;
        };
        if quote != '\'' && quote != '"' && quote != '`' {
            continue;
        }
        let body = &after[quote.len_utf8()..];
        let end = body.find(quote).expect("close quote");
        found.push((
            u32::try_from(start).expect("offset"),
            body[..end].to_string(),
        ));
    }
    found
}

fn require_imports(facts: &ImportFacts) -> Vec<(String, ImportKind, bool)> {
    let mut imports: Vec<_> = facts
        .imports
        .iter()
        .filter(|import| {
            matches!(
                import.kind,
                ImportKind::Require | ImportKind::RequireResolve
            )
        })
        .map(|import| (import.specifier.clone(), import.kind, import.computed))
        .collect();
    imports.sort_by(|left, right| left.0.cmp(&right.0));
    imports
}

#[test]
fn a_created_require_records_literal_calls_at_top_level_and_inside_functions() {
    let facts = facts_of(LOADS, SourceType::ts());
    let expected = require_literals(LOADS);
    assert_eq!(expected.len(), 4, "{expected:?}");
    assert_eq!(facts.namespaces.literal_import_spans, expected);
    assert!(
        require_imports(&facts).is_empty(),
        "a literal created-require call records a span and keeps its call site"
    );
}

#[test]
fn a_non_literal_created_require_is_kept_out_of_shared_imports() {
    let facts = facts_of(DYNAMIC, SourceType::ts());
    assert!(facts.namespaces.literal_import_spans.is_empty());
    assert!(facts.namespaces.computed_created_require);
    assert!(require_imports(&facts).is_empty());
}

#[test]
fn renamed_namespace_and_default_factories_record_literal_loads() {
    let facts = facts_of(FACTORIES, SourceType::ts());
    assert_eq!(
        facts.namespaces.literal_import_spans,
        [
            (offset(FACTORIES, "load("), "../widget.cjs".to_string()),
            (
                offset(FACTORIES, "again.resolve("),
                "../item.cjs".to_string()
            ),
            (
                offset(FACTORIES, "defaults("),
                "../defaults.cjs".to_string()
            ),
        ]
    );
    assert!(require_imports(&facts).is_empty());
}

#[test]
fn only_a_const_from_a_runtime_factory_records_a_literal_load() {
    let facts = facts_of(NON_FACTORIES, SourceType::ts());
    assert_eq!(
        facts.namespaces.literal_import_spans,
        [(
            offset(NON_FACTORIES, "direct("),
            "../direct.cjs".to_string()
        )]
    );
    assert!(require_imports(&facts).is_empty());
    assert!(facts
        .imports
        .iter()
        .any(|import| import.side_effect_only && import.specifier == "node:module"));
}

#[test]
fn an_outer_created_require_still_records_after_an_inner_shadow() {
    let facts = facts_of(SHADOWED, SourceType::ts());
    assert_eq!(
        facts.namespaces.literal_import_spans,
        require_literals(SHADOWED)
            .into_iter()
            .filter(|(_, specifier)| specifier == "../item.cjs")
            .collect::<Vec<_>>()
    );
}

#[test]
fn global_require_records_both_import_edges_and_call_sites() {
    let facts = facts_of(GLOBAL, SourceType::cjs());
    assert_eq!(
        require_imports(&facts),
        [
            ("../item.cjs".to_string(), ImportKind::RequireResolve, false),
            ("../widget.cjs".to_string(), ImportKind::Require, false),
        ]
    );
    let calls: Vec<_> = facts
        .function_calls
        .iter()
        .map(|call| {
            (
                call.callee.as_str(),
                call.caller.as_deref(),
                call.offset,
                call.static_arg.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        calls,
        [
            (
                "require",
                Some("loadWidget"),
                offset(GLOBAL, "require("),
                Some("../widget.cjs"),
            ),
            (
                "require.resolve",
                Some("loadWidget"),
                offset(GLOBAL, "require.resolve("),
                Some("../item.cjs"),
            ),
        ]
    );
}

fn offset(source: &str, token: &str) -> u32 {
    u32::try_from(source.find(token).expect("call in fixture")).expect("offset")
}
