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

fn facts_of(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        source,
        SourceType::from_path("created.mts").expect("mts source type"),
    )
    .parse();
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
    let facts = facts_of(LOADS);
    let expected = require_literals(LOADS);
    assert_eq!(expected.len(), 4, "{expected:?}");
    assert_eq!(facts.namespaces.literal_import_spans, expected);
    let mut imports = require_imports(&facts);
    imports.retain(|(_, _, computed)| !computed);
    assert_eq!(
        imports,
        vec![
            (
                "../early-widget.cjs".to_string(),
                ImportKind::Require,
                false
            ),
            (
                "../item-defaults.cjs".to_string(),
                ImportKind::Require,
                false
            ),
            ("../order-table.cjs".to_string(), ImportKind::Require, false),
            (
                "../widget-settings.cjs".to_string(),
                ImportKind::Require,
                false
            ),
        ]
    );
}

#[test]
fn a_non_literal_created_require_stays_computed_without_a_span() {
    let facts = facts_of(DYNAMIC);
    assert!(facts.namespaces.literal_import_spans.is_empty());
    assert_eq!(
        require_imports(&facts),
        vec![("specifier".to_string(), ImportKind::Require, true)]
    );
}

#[test]
fn renamed_and_namespace_factories_record_require_and_resolve() {
    let facts = facts_of(concat!(
        "import { createRequire as makeRequire } from 'node:module'\n",
        "import * as nodeModule from 'module'\n",
        "const load = makeRequire(import.meta.url)\n",
        "const again = nodeModule.createRequire(import.meta.url)\n",
        "load('../widget.cjs')\n",
        "again.resolve('../item.cjs')\n",
    ));
    assert_eq!(
        require_imports(&facts),
        vec![
            ("../item.cjs".to_string(), ImportKind::RequireResolve, false),
            ("../widget.cjs".to_string(), ImportKind::Require, false),
        ]
    );
}

#[test]
fn mutable_aliased_and_shadowed_factories_do_not_record_a_require() {
    for source in [
        concat!(
            "import { createRequire } from 'node:module'\n",
            "let require = createRequire(import.meta.url)\n",
            "require('../widget.cjs')\n",
        ),
        concat!(
            "import { createRequire } from 'node:module'\n",
            "const make = createRequire\n",
            "const require = make(import.meta.url)\n",
            "require('../widget.cjs')\n",
        ),
        concat!(
            "import { createRequire } from 'node:module'\n",
            "const require = createRequire(import.meta.url)\n",
            "export function load() {\n",
            "  const require = (specifier: string) => specifier\n",
            "  return require('../widget.cjs')\n",
            "}\n",
            "require('../item.cjs')\n",
        ),
        concat!(
            "import type { createRequire } from 'node:module'\n",
            "const require = createRequire(import.meta.url)\n",
            "require('../widget.cjs')\n",
        ),
    ] {
        let facts = facts_of(source);
        let spans = facts.namespaces.literal_import_spans;
        assert!(
            spans
                .iter()
                .all(|(_, specifier)| specifier != "../widget.cjs"),
            "{source}\n{spans:?}"
        );
    }
}

#[test]
fn an_outer_created_require_still_records_after_an_inner_shadow() {
    let source = concat!(
        "import { createRequire } from 'node:module'\n",
        "const require = createRequire(import.meta.url)\n",
        "export function load() {\n",
        "  const require = (specifier: string) => specifier\n",
        "  return require('../widget.cjs')\n",
        "}\n",
        "require('../item.cjs')\n",
    );
    let facts = facts_of(source);
    assert_eq!(
        facts.namespaces.literal_import_spans,
        require_literals(source)
            .into_iter()
            .filter(|(_, specifier)| specifier == "../item.cjs")
            .collect::<Vec<_>>()
    );
}
