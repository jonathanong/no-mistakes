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
