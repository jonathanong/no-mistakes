use no_mistakes::ast;
use no_mistakes::fetch::import_shape::is_runtime_import;
use no_mistakes::fetch::imports::{
    collect_identifier_references, collect_imports, collect_imports_from_program,
    collect_runtime_imports_from_program,
};
use oxc_ast::ast::Statement;
use std::collections::HashMap;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-fetches")
        .join(name)
        .join("fixture")
}

#[test]
fn test_collect_imports_reuses_cached_imports() {
    let file = fixture("collect-imports-cached").join("pkg/index.ts");

    let mut import_cache = HashMap::new();
    let first = collect_imports(&file, &mut import_cache).unwrap();
    let second = collect_imports(&file, &mut import_cache).unwrap();
    assert_eq!(first, second);
}

#[test]
fn test_collect_imports_from_program_reuses_cached_value() {
    let file = fixture("collect-imports-from-program-cached").join("main.ts");

    let abs_path = file.canonicalize().unwrap();
    let source = std::fs::read_to_string(&abs_path).unwrap();
    let mut import_cache = HashMap::new();
    let mut from_source = false;
    ast::with_program(&abs_path, &source, |program, _source| {
        let first = collect_imports_from_program(&abs_path, program, &mut import_cache);
        let second = collect_imports_from_program(&abs_path, program, &mut import_cache);
        assert_eq!(first, second);
        assert_eq!(first.len(), 1);
        from_source = !first.is_empty();
    })
    .unwrap();
    assert!(from_source);
}

#[test]
fn test_collect_imports_filters_runtime_and_type_only_imports_exports() {
    let file = fixture("collect-imports-runtime-and-type-only").join("pkg/index.ts");

    let mut import_cache = HashMap::new();
    let imports = collect_imports(&file, &mut import_cache).unwrap();
    assert_eq!(imports.len(), 3);
    assert!(imports.iter().any(|path| path.ends_with("side-effect.ts")));
    assert!(imports.iter().any(|path| path.ends_with("runtime.ts")));
    assert!(imports.iter().any(|path| path.ends_with("runtime-all.ts")));
    assert!(!imports.iter().any(|path| path.ends_with("types.ts")));
}

#[test]
fn test_collect_runtime_imports_from_program_follows_used_runtime_imports_only() {
    let main_file = fixture("collect-runtime-imports-used-only").join("main.ts");
    let source = std::fs::read_to_string(&main_file).unwrap();
    let allocator = oxc_allocator::Allocator::default();
    let source_type = oxc_span::SourceType::from_path(std::path::Path::new("main.ts")).unwrap();
    let parsed = oxc_parser::Parser::new(&allocator, &source, source_type).parse();
    let referenced_identifiers = collect_identifier_references(&parsed.program);
    let imports =
        collect_runtime_imports_from_program(&main_file, &parsed.program, &referenced_identifiers);

    assert_eq!(imports.len(), 4);
    assert!(imports.iter().any(|path| path.ends_with("side-effect.ts")));
    assert!(imports.iter().any(|path| path.ends_with("named.ts")));
    assert!(imports.iter().any(|path| path.ends_with("default.ts")));
    assert!(imports.iter().any(|path| path.ends_with("namespace.ts")));
    assert!(!imports.iter().any(|path| path.ends_with("only-unused.ts")));
    assert!(!imports.iter().any(|path| path.ends_with("types.ts")));
}

#[test]
fn test_is_runtime_import_variants() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "
            const constant = 1;
            import type { Foo } from './foo';
            import {} from './empty';
            import { type Bar } from './bar';
            import { Baz } from './baz';
            import Widget, { type Props } from './widget';
            import * as all from './all';
        ";
    let source_type = oxc_span::SourceType::from_path(std::path::Path::new("test.ts")).unwrap();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "import parser errors: {:?}",
        parsed.diagnostics
    );
    let imports = parsed
        .program
        .body
        .iter()
        .filter_map(|stmt| match stmt {
            Statement::ImportDeclaration(import) => Some(import),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(imports.len(), 6);
    assert!(!is_runtime_import(imports[0]));
    assert!(is_runtime_import(imports[1]));
    assert!(!is_runtime_import(imports[2]));
    assert!(is_runtime_import(imports[3]));
    assert!(is_runtime_import(imports[4]));
    assert!(is_runtime_import(imports[5]));
}
