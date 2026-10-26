use super::*;
use oxc_parser::Parser;

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

#[test]
fn call_facts_preserve_string_literal_export_names() {
    let source = "export { handler as \"call-handler\" } from './target.mts';";
    let allocator = oxc_allocator::Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    assert!(facts.exported_bindings.iter().any(|binding| {
        binding.specifier.as_deref() == Some("./target.mts")
            && binding.local == "handler"
            && binding.exported == "call-handler"
    }));
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
    assert!(!imports[0].computed);
}

#[test]
fn non_literal_dynamic_import_is_computed() {
    let imports = ts_extractor()
        .extract("const m = await import(moduleName);")
        .unwrap();
    assert_eq!(specs(&imports), vec!["moduleName"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Dynamic]);
    assert!(imports[0].computed);
}

#[test]
fn interpolated_dynamic_import_is_computed() {
    let imports = ts_extractor()
        .extract("const m = await import(`./${name}`);")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./${}"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Dynamic]);
    assert!(imports[0].computed);
}

#[test]
fn concatenated_dynamic_import_is_computed() {
    let imports = ts_extractor()
        .extract(r#"const m = import("./" + name);"#)
        .unwrap();
    assert_eq!(specs(&imports), vec!["<computed>"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Dynamic]);
    assert!(imports[0].computed);
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
fn non_literal_require_call_is_computed() {
    let imports = ts_extractor()
        .extract("const mod = require(moduleName);")
        .unwrap();
    assert_eq!(specs(&imports), vec!["moduleName"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Require]);
    assert!(imports[0].computed);
}

#[test]
fn spread_require_call_is_computed() {
    let imports = ts_extractor().extract("require(...mods);").unwrap();
    assert_eq!(specs(&imports), vec!["<computed>"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Require]);
    assert!(imports[0].computed);
}

#[test]
fn require_without_arguments_is_ignored() {
    let imports = ts_extractor().extract("require();").unwrap();
    assert!(imports.is_empty());
}

#[test]
fn exported_dynamic_loader_literal_is_not_computed() {
    let imports = ts_extractor()
        .extract("export const Lazy = dynamic(() => import('./Foo'));")
        .unwrap();
    assert!(imports
        .iter()
        .any(|imp| imp.specifier == "./Foo" && !imp.computed && imp.kind == ImportKind::Dynamic));
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

mod symbol_references;

#[path = "tests/accessor_kind_regressions.rs"]
mod accessor_kind_regressions;
#[path = "tests/call_apply_regressions.rs"]
mod call_apply_regressions;
mod call_binding_metadata;
#[path = "tests/call_binding_predeclaration_regressions.rs"]
mod call_binding_predeclaration_regressions;
#[path = "tests/call_binding_reassignment_scope.rs"]
mod call_binding_reassignment_scope;
#[path = "tests/call_binding_regressions.rs"]
mod call_binding_regressions;
#[path = "tests/call_binding_shadow_regressions.rs"]
mod call_binding_shadow_regressions;
#[path = "tests/callable_alias_regressions.rs"]
mod callable_alias_regressions;
#[path = "tests/callable_alias_tdz.rs"]
mod callable_alias_tdz;
#[path = "tests/class_and_overload_regressions.rs"]
mod class_and_overload_regressions;
#[path = "tests/coverage_shapes.rs"]
mod coverage_shapes;
#[path = "tests/dynamic_callee_shapes.rs"]
mod dynamic_callee_shapes;
#[path = "tests/eager_requires.rs"]
mod eager_requires;
#[cfg(test)]
#[path = "tests/namespace_facts.rs"]
mod namespace_facts;
#[cfg(test)]
#[path = "tests/namespace_locals.rs"]
mod namespace_locals;
#[cfg(test)]
#[path = "tests/namespace_uses.rs"]
mod namespace_uses;
#[path = "tests/object_spread_regressions.rs"]
mod object_spread_regressions;
#[path = "tests/static_block_regressions.rs"]
mod static_block_regressions;
#[path = "tests/this_member_regressions.rs"]
mod this_member_regressions;
