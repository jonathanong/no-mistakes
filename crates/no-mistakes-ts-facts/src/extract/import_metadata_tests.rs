use super::*;
use oxc_parser::Parser;

fn ts_extractor() -> ImportExtractor {
    ImportExtractor::for_typescript().unwrap()
}

fn specs(imports: &[ExtractedImport]) -> Vec<&str> {
    imports.iter().map(|i| i.specifier.as_str()).collect()
}

fn kinds(imports: &[ExtractedImport]) -> Vec<ImportKind> {
    imports.iter().map(|i| i.kind).collect()
}

#[test]
fn extracts_require_resolve_call() {
    let imports = ts_extractor()
        .extract("const path = require.resolve('@scope/pkg/register');")
        .unwrap();
    assert_eq!(specs(&imports), vec!["@scope/pkg/register"]);
    assert_eq!(kinds(&imports), vec![ImportKind::RequireResolve]);
}

#[test]
fn expression_free_require_resolve_template_is_not_computed() {
    let imports = ts_extractor()
        .extract("const path = require.resolve(`@scope/pkg/register`);")
        .unwrap();
    assert_eq!(specs(&imports), vec!["@scope/pkg/register"]);
    assert_eq!(kinds(&imports), vec![ImportKind::RequireResolve]);
    assert!(!imports[0].computed);
}

#[test]
fn expression_free_require_template_is_not_computed() {
    let imports = ts_extractor()
        .extract("const mod = require(`./cjs.js`);")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./cjs.js"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Require]);
    assert!(!imports[0].computed);
}

#[test]
fn interpolated_require_template_is_computed() {
    let imports = ts_extractor()
        .extract("const mod = require(`./${name}`);")
        .unwrap();
    assert_eq!(specs(&imports), vec!["./${}"]);
    assert_eq!(kinds(&imports), vec![ImportKind::Require]);
    assert!(imports[0].computed);
}

#[test]
fn non_literal_require_resolve_call_is_computed() {
    let imports = ts_extractor()
        .extract("const path = require.resolve(moduleName);")
        .unwrap();
    assert_eq!(specs(&imports), vec!["moduleName"]);
    assert_eq!(kinds(&imports), vec![ImportKind::RequireResolve]);
    assert!(imports[0].computed);
}

#[test]
fn argumentless_require_resolve_call_is_ignored() {
    let imports = ts_extractor()
        .extract("const path = require.resolve();")
        .unwrap();
    assert!(imports.is_empty());
}

#[test]
fn imported_require_factory_does_not_create_commonjs_edges() {
    for file in ["imported-require.test.ts", "imported-require-late.test.ts"] {
        let path = crate::test_support::rule_fixture_root("test-no-live-repository-scan")
            .join("tests")
            .join(file);
        let source = std::fs::read_to_string(path).unwrap();
        let imports = ts_extractor().extract(&source).unwrap();
        assert_eq!(specs(&imports), vec!["../helpers/require-factory"]);
        assert_eq!(kinds(&imports), vec![ImportKind::Static]);
    }
}

#[test]
fn erased_type_import_does_not_hide_commonjs_loader() {
    let path = crate::test_support::rule_fixture_root("test-no-live-repository-scan")
        .join("helpers/type-only-require.ts");
    let source = std::fs::read_to_string(path).unwrap();
    let imports = ts_extractor().extract(&source).unwrap();
    assert!(imports
        .iter()
        .any(|import| { import.kind == ImportKind::Require && import.specifier == "node:fs" }));
}

#[test]
fn records_lines_side_effects_and_reexports() {
    let imports = ts_extractor()
        .extract("import 'polyfill';\nexport { helper } from '@scope/pkg/helpers';")
        .unwrap();
    assert_eq!(imports[0].line, 1);
    assert!(imports[0].side_effect_only);
    assert!(!imports[0].re_export);
    assert_eq!(imports[1].line, 2);
    assert!(!imports[1].side_effect_only);
    assert!(imports[1].re_export);
}

#[test]
fn source_less_program_extraction_defaults_import_lines_to_one() {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = Parser::new(
        &allocator,
        "\n\nconst resolved = require.resolve('pkg');",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&parsed.program);

    assert_eq!(facts.imports[0].line, 1);
}
