use super::*;

fn ts_extractor() -> ImportExtractor {
    ImportExtractor::for_typescript().unwrap()
}

fn tsx_extractor() -> ImportExtractor {
    ImportExtractor::for_tsx().unwrap()
}

fn specs(imports: &[ExtractedImport]) -> Vec<&str> {
    imports.iter().map(|i| i.specifier.as_str()).collect()
}

fn kinds(imports: &[ExtractedImport]) -> Vec<ImportKind> {
    imports.iter().map(|i| i.kind).collect()
}

// ── Basic import forms ──────────────────────────────────────────────

#[test]
fn extracts_default_import() {
    let imports = ts_extractor()
        .extract("import foo from './foo.mts';")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./foo.mts"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Static]);
}

#[test]
fn extract_imports_from_program_wraps_full_import_fact_extraction() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "import foo from './foo.mts';";
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();

    let imports = extract_imports_from_program(&parsed.program);

    assert_eq!(specs(&imports), vec!["./foo.mts"]);
}

#[test]
fn extracts_named_import() {
    let imports = ts_extractor()
        .extract("import { bar } from './bar.mts';")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./bar.mts"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Static]);
}

#[test]
fn extracts_side_effect_import() {
    let imports = ts_extractor().extract("import './polyfill.mts';").unwrap();
    assert_eq!(specs(&imports), vec!["./polyfill.mts"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Static]);
}

#[test]
fn extracts_alias_import() {
    let imports = ts_extractor()
        .extract("import { x } from '@utils/helpers';")
        .unwrap();
    assert_eq!(specs(&imports), vec!["@utils/helpers"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Static]);
}

#[test]
fn extracts_star_export() {
    let imports = ts_extractor()
        .extract("export * from './module.mts';")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./module.mts"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Static]);
}

#[test]
fn extracts_type_star_export() {
    let imports = ts_extractor()
        .extract("export type * from './types.mts';")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./types.mts"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Type]);
}

#[test]
fn extracts_named_reexport() {
    let imports = ts_extractor()
        .extract("export { foo } from './foo.mts';")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./foo.mts"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Static]);
}

// ── Type-only forms ─────────────────────────────────────────────────

#[test]
fn extracts_type_import() {
    let imports = ts_extractor()
        .extract("import type { Foo } from './types.mts';")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./types.mts"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Type]);
}

#[test]
fn extracts_type_reexport() {
    let imports = ts_extractor()
        .extract("export type { Foo } from './types.mts';")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./types.mts"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Type]);
}

#[test]
fn extracts_inline_type_only_import_as_type() {
    let imports = ts_extractor()
        .extract("import { type X } from './types.mts';")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./types.mts"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Type]);
}

#[test]
fn mixed_inline_type_import_is_static() {
    let imports = ts_extractor()
        .extract("import { type X, Y } from './mixed.mts';")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./mixed.mts"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Static]);
}

#[test]
fn extracts_inline_type_only_reexport_as_type() {
    let imports = ts_extractor()
        .extract("export { type X } from './types.mts';")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./types.mts"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Type]);
}

#[test]
fn mixed_inline_type_reexport_is_static() {
    let imports = ts_extractor()
        .extract("export { type X, Y } from './mixed.mts';")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./mixed.mts"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Static]);
}

#[test]
fn extracts_ts_import_type_as_type() {
    let imports = ts_extractor()
        .extract("type User = import('./types.mts').User;")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./types.mts"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Type]);
}

// ── Runtime import forms ────────────────────────────────────────────

#[test]
fn extracts_dynamic_import() {
    let imports = ts_extractor()
        .extract("const m = await import('./dyn.mts');")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./dyn.mts"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Dynamic]);
}

#[test]
fn non_literal_dynamic_import_is_ignored() {
    let imports = ts_extractor()
        .extract("const m = await import(moduleName);")
        .unwrap();
    assert!(imports.is_empty());
}

#[test]
fn extracts_require_call() {
    let imports = ts_extractor()
        .extract("const mod = require('./cjs.js');")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./cjs.js"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Require]);
}

#[test]
fn non_literal_require_call_is_ignored() {
    let imports = ts_extractor()
        .extract("const mod = require(moduleName);")
        .unwrap();
    assert!(imports.is_empty());
}

#[test]
fn require_without_arguments_is_ignored() {
    let imports = ts_extractor().extract("require();").unwrap();
    assert!(imports.is_empty());
}

// ── General behavior ────────────────────────────────────────────────

#[test]
fn extracts_multiple_imports() {
    let src = "import a from './a.mts';\nimport b from './b.mts';\n";
    let imports = ts_extractor().extract(src).unwrap();
    assert_eq!(specs(&imports), vec!["./a.mts", "./b.mts"]);
    assert_eq!(
        kinds(&imports),
        vec![ImportKind::Static, ImportKind::Static]
    );
}

#[test]
fn empty_source_returns_empty() {
    let imports = ts_extractor().extract("").unwrap();
    assert!(imports.is_empty());
}

#[test]
fn no_imports_returns_empty() {
    let imports = ts_extractor()
        .extract("const x = 1;\nexport { x };\n")
        .unwrap();
    assert!(imports.is_empty());
}

#[test]
fn type_and_value_import_same_module_tagged_independently() {
    let src = "import type { A } from './utils.mts';\nimport { b } from './utils.mts';\n";
    let imports = ts_extractor().extract(src).unwrap();
    assert_eq!(imports.len(), 2);
    assert_eq!(kinds(&imports), vec![ImportKind::Type, ImportKind::Static]);
}

// ── TSX ─────────────────────────────────────────────────────────────

#[test]
fn tsx_extracts_imports() {
    let src = "import React from 'react';\nimport { Foo } from './Foo.tsx';\n";
    let imports = tsx_extractor().extract(src).unwrap();
    assert_eq!(specs(&imports), vec!["react", "./Foo.tsx"]);
}

// ── File-based regression for mixed type+value imports from same module ──

#[test]
fn fixture_mixed_type_import_file() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/mixed-type-import/fixture/importer.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let imports = ts_extractor().extract(&source).unwrap();
    assert_eq!(kinds(&imports), vec![ImportKind::Type, ImportKind::Static]);
    assert_eq!(specs(&imports), vec!["./types.mts", "./types.mts"]);
}

#[test]
fn fixture_function_expression_import_tracks_named_scope() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/function-expression.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(facts.imports[0].specifier, "./loaded.mts");
    assert_eq!(facts.imports[0].function_scope.as_deref(), Some("loader"));
    assert_eq!(facts.function_calls[0].callee, "loader");
}

#[test]
fn static_member_call_records_namespace_member_name() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "function run() { dates.parseDate(); }",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert!(facts
        .function_calls
        .iter()
        .any(|call| { call.caller.as_deref() == Some("run") && call.callee == "dates.parseDate" }));
}

#[test]
fn exported_declarations_record_identifier_and_type_references() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import { alpha } from './source.mts';\nimport type { SourceShape } from './source.mts';\nexport const alias: SourceShape = alpha;\nexport type AliasShape = SourceShape;",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert!(facts
        .symbol_references
        .iter()
        .any(|call| { call.caller.as_deref() == Some("alias") && call.callee == "alpha" }));
    assert!(facts
        .symbol_references
        .iter()
        .any(|call| { call.caller.as_deref() == Some("alias") && call.callee == "SourceShape" }));
    assert!(facts.symbol_references.iter().any(|call| {
        call.caller.as_deref() == Some("AliasShape") && call.callee == "SourceShape"
    }));
}

#[test]
fn function_parameters_shadow_imported_symbol_references() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import { alpha, beta, gamma, rest } from './source.mts';
         export function run({ alpha }: { alpha: number }, [beta]: number[], gamma = 1, ...rest: number[]) {
           return alpha + beta + gamma + rest.length;
         }",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);
    let callees: Vec<_> = facts
        .symbol_references
        .iter()
        .filter(|call| call.caller.as_deref() == Some("run"))
        .map(|call| call.callee.as_str())
        .collect();

    assert!(!callees.contains(&"alpha"));
    assert!(!callees.contains(&"beta"));
    assert!(!callees.contains(&"gamma"));
    assert!(!callees.contains(&"rest"));
}

#[test]
fn fixture_assignment_pattern_shadows_imported_calls() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/assignment-pattern-shadow.mts",
    );
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);
    let calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.caller.as_deref() == Some("run") && call.callee == "loaded")
        .collect();

    assert!(calls.is_empty());
    assert!(!facts
        .symbol_references
        .iter()
        .any(|call| call.caller.as_deref() == Some("run") && call.callee == "loaded"));
}

#[test]
fn fixture_anonymous_variable_function_records_anonymous_scope() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/anonymous-variable-function.mts",
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
fn fixture_catch_parameter_shadows_only_catch_block_references() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/catch-shadow.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);
    let calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.caller.as_deref() == Some("run") && call.callee == "loaded")
        .collect();

    assert_eq!(calls.len(), 3);
}

#[test]
fn fixture_var_binding_shadows_outside_block_references() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/var-shadow.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);
    let calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.caller.as_deref() == Some("run") && call.callee == "loaded")
        .collect();

    assert_eq!(calls.len(), 1);
    assert!(!facts
        .symbol_references
        .iter()
        .any(|call| call.caller.as_deref() == Some("run") && call.callee == "loaded"));
}

#[test]
fn fixture_function_expression_falls_back_to_inner_name() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/destructured-function-expression.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(
        facts.imports[0].function_scope.as_deref(),
        Some("namedLoader")
    );
}

#[test]
fn local_string_named_export_is_not_marked_as_exported_function() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/local-string-named-export.mts",
    );
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert!(facts.exported_functions.is_empty());
}

#[test]
fn exported_functions_are_sorted() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/exported-functions-order.mts",
    );
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.exported_functions, vec!["alpha", "middle", "zeta"]);
}

#[test]
fn function_expression_declarator_binding_pattern_is_visited() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/function-expression-binding-pattern.mts",
    );
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(facts.imports[0].specifier, "./cfg.mts");
    assert_eq!(facts.imports[0].function_scope, None);
}

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
fn fixture_computed_function_keys_are_visited_outside_scopes() {
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
            ("./key.mts", None),
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
