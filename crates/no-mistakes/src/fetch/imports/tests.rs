use super::*;
use crate::ast;
use std::collections::{HashMap, HashSet};

#[test]
fn test_collect_runtime_imports_from_program() {
    let source = r#"
        import { A, B } from './file_to_import';
        import { C } from './unused_import';
        import type { D } from './type_import';

        console.log(A, B);
    "#;

    let temp_dir = tempfile::tempdir().unwrap();
    let pwd = temp_dir.path();
    let path = pwd.join("dummy.ts");

    // Create dummy files so resolve_import returns Some
    std::fs::write(pwd.join("file_to_import.ts"), "").unwrap();
    std::fs::write(pwd.join("unused_import.ts"), "").unwrap();

    ast::with_program(&path, source, |program, _| {
        let mut referenced_identifiers = HashSet::new();
        referenced_identifiers.insert("A".to_string());
        referenced_identifiers.insert("B".to_string());

        let imports = collect_runtime_imports_from_program(&path, program, &referenced_identifiers);

        // We expect 1 import (the one for file_to_import) to be returned.
        // Type imports and unused imports should be filtered out.
        assert_eq!(imports.len(), 1);
        assert!(imports[0].to_string_lossy().contains("file_to_import"));
    })
    .unwrap();
}

#[test]
fn test_collect_imports_from_program() {
    let source = r#"
        import { A } from './runtime_import';
        import type { B } from './type_import';
        export { C } from './runtime_export';
        export type { D } from './type_export';
        export * from './runtime_export_all';
        export type * from './type_export_all';
    "#;

    let temp_dir = tempfile::tempdir().unwrap();
    let pwd = temp_dir.path();
    let path = pwd.join("dummy.ts");

    // Create dummy files so resolve_import returns Some
    std::fs::write(pwd.join("runtime_import.ts"), "").unwrap();
    std::fs::write(pwd.join("type_import.ts"), "").unwrap();
    std::fs::write(pwd.join("runtime_export.ts"), "").unwrap();
    std::fs::write(pwd.join("type_export.ts"), "").unwrap();
    std::fs::write(pwd.join("runtime_export_all.ts"), "").unwrap();
    std::fs::write(pwd.join("type_export_all.ts"), "").unwrap();

    let mut import_cache = HashMap::new();

    ast::with_program(&path, source, |program, _| {
        let imports = collect_imports_from_program(&path, program, &mut import_cache);

        assert_eq!(imports.len(), 3);

        let import_strs: Vec<_> = imports
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        assert!(import_strs.iter().any(|s| s.contains("runtime_import")));
        assert!(import_strs.iter().any(|s| s.contains("runtime_export")));
        assert!(import_strs.iter().any(|s| s.contains("runtime_export_all")));

        // Test cache hit
        let cached_imports = collect_imports_from_program(&path, program, &mut import_cache);
        assert_eq!(cached_imports, imports);
    })
    .unwrap();
}

#[test]
fn test_is_import_used() {
    let source = r#"
        import './side_effect';
        import DefaultImport from './default';
        import * as NamespaceImport from './namespace';
        import { NamedImport } from './named';
        import { UnusedImport } from './unused';
        import { Used, Unused2 } from './mixed';
    "#;

    let path = std::path::PathBuf::from("dummy.ts");

    ast::with_program(&path, source, |program, _| {
        let mut referenced_identifiers = HashSet::new();
        referenced_identifiers.insert("DefaultImport".to_string());
        referenced_identifiers.insert("NamespaceImport".to_string());
        referenced_identifiers.insert("NamedImport".to_string());
        referenced_identifiers.insert("Used".to_string());

        let imports: Vec<_> = program
            .body
            .iter()
            .filter_map(|stmt| {
                if let oxc_ast::ast::Statement::ImportDeclaration(import) = stmt {
                    Some(import)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(imports.len(), 6);

        // 1. Side-effect import (no specifiers) -> always considered used
        assert!(
            is_import_used(imports[0], &referenced_identifiers),
            "Side-effect import should be used"
        );

        // 2. Default import -> used
        assert!(
            is_import_used(imports[1], &referenced_identifiers),
            "Default import should be used"
        );

        // 3. Namespace import -> used
        assert!(
            is_import_used(imports[2], &referenced_identifiers),
            "Namespace import should be used"
        );

        // 4. Named import -> used
        assert!(
            is_import_used(imports[3], &referenced_identifiers),
            "Named import should be used"
        );

        // 5. Unused import -> not used
        assert!(
            !is_import_used(imports[4], &referenced_identifiers),
            "Unused import should NOT be used"
        );

        // 6. Mixed import (one used, one unused) -> used
        assert!(
            is_import_used(imports[5], &referenced_identifiers),
            "Mixed import should be used"
        );
    })
    .unwrap();
}

#[test]
fn pass4a_ignored_import_candidate_does_not_shadow_visible_route_fallback() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4a-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let route = root.join("fetch/nested/route.ts");
    let target = root.join("fetch/nested/target.ts");
    let visible_files = crate::codebase::ts_source::discover_visible_paths(&root)
        .into_iter()
        .collect();

    let reaches = crate::fetch::import_routes::route_reaches_target_from_visible(
        &route,
        &target,
        &mut HashSet::new(),
        &mut HashMap::new(),
        &visible_files,
    )
    .unwrap();

    assert!(reaches);
}
