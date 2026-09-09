use super::*;
use oxc_parser::Parser;

#[test]
fn inline_function_like_members_record_parameters_and_type_parameters() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import { loaded, SourceShape } from './source.mts';
         const handlers = {
           method<T extends SourceShape>(value: T = loaded()) { return value; },
           expression: function<T extends SourceShape>(value: T = loaded()) { return value; },
           arrow: <T extends SourceShape>(value: T = loaded()) => value,
         };
         export default function<T extends SourceShape>(value: T = loaded()) { return value; }",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);
    let scopes: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.callee == "loaded")
        .map(|call| call.caller.as_deref())
        .collect();

    assert_eq!(
        scopes,
        vec![
            Some("handlers/method"),
            Some("handlers/expression"),
            Some("handlers/arrow"),
            Some("default")
        ]
    );
    assert!(facts
        .symbol_references
        .iter()
        .any(|call| call.caller.as_deref() == Some("handlers/method")
            && call.callee == "SourceShape"));
}

#[test]
fn generic_type_parameters_shadow_imported_type_references() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import type { SourceShape } from './source.mts';\nexport type Box<SourceShape> = SourceShape;",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.symbol_references, Vec::<FunctionCall>::new());
}

#[test]
fn fixture_default_function_expression_uses_expression_scope() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/default-function-expression.mts",
    );
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(facts.imports[0].specifier, "./loaded.mts");
    assert_eq!(facts.imports[0].function_scope.as_deref(), Some("default"));
}

#[test]
fn fixture_nested_class_uses_default_walk() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/nested-class.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(facts.imports[0].specifier, "./loaded.mts");
    assert_eq!(
        facts.imports[0].function_scope.as_deref(),
        Some("outer/run")
    );
}

#[test]
fn fixture_local_enum_uses_default_walk() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/local-enum.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(facts.imports[0].specifier, "./source.mts");
    assert_eq!(facts.symbol_references.len(), 1);
    assert_eq!(facts.symbol_references[0].caller, None);
    assert_eq!(facts.symbol_references[0].callee, "alpha");
}

#[test]
fn destructuring_array_defaults_are_scoped_to_each_binding() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import { alpha } from './source.mts';\nconst list = [];\nexport const [api = alpha] = list;",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert!(facts
        .symbol_references
        .iter()
        .any(|call| call.caller.as_deref() == Some("api") && call.callee == "alpha"));
}

#[test]
fn default_expression_helpers_cover_parenthesized_forms_and_static_args() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import { alpha } from './source.mts';
         export default ((function() { alpha(); }));
         export const template = () => fetch(`/api/users/42`);",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.caller.as_deref() == Some("default") && call.callee == "alpha"));
    assert!(facts.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("template")
            && call.callee == "fetch"
            && call.static_arg.as_deref() == Some("/api/users/42")
    }));
    let ret = Parser::new(
        &allocator,
        "import { alpha } from './source.mts';\nexport default (({ run() { alpha(); } }));",
        SourceType::ts(),
    )
    .parse();
    let facts = extract_import_facts_from_program(&ret.program);
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.caller.as_deref() == Some("default") && call.callee == "run"));

    let ret = Parser::new(
        &allocator,
        "import { alpha } from './source.mts';\nexport default function() { alpha(); }",
        SourceType::ts(),
    )
    .parse();
    let facts = extract_import_facts_from_program(&ret.program);
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.caller.as_deref() == Some("default") && call.callee == "alpha"));
}

#[test]
fn import_collector_nests_interned_keys_on_fxhash() {
    let extract = include_str!("../extract.rs");
    let collector = include_str!("../extract_collector.rs");
    let maps = include_str!("../extract_collector_maps.rs");
    let scopes = include_str!("../extract_collector_scopes.rs");

    assert!(
        extract.contains("use crate::fx::{fx_map, fx_set, FxHashMap, FxHashSet}"),
        "extract collector must import crate::fx maps"
    );
    assert!(
        !extract.contains("std::collections::HashMap"),
        "extract collector must not import SipHash HashMap"
    );
    assert!(
        !extract.contains("std::collections::HashSet"),
        "extract collector must not import SipHash HashSet"
    );
    assert!(
        collector.contains("include!(\"extract_collector_maps.rs\")"),
        "nested FxHash helpers must stay in extract_collector_maps.rs"
    );
    assert!(
        collector.contains("callable_bindings: Vec<FxHashMap<String, CallableId>>"),
        "lexical bindings must nest per scope so lookups use &str"
    );
    assert!(
        collector.contains(
            "class_member_callable_ids: FxHashMap<CallableId, FxHashMap<String, CallableId>>"
        ),
        "class members must nest by owner so (class, member) probes skip a linear scan"
    );
    assert!(
        !collector.contains("HashMap<(usize, String)"),
        "do not key collector maps on allocated (scope, String) tuples"
    );
    assert!(
        maps.contains("fn scope_map_get"),
        "name lookups must go through borrowed &str helpers"
    );
    assert!(
        !scopes.contains("HashSet::new()"),
        "rustc-hash 2 FxHashSet has no new(); use fx_set()"
    );
}
