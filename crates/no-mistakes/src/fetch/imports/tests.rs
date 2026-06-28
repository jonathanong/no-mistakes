use super::*;
use crate::ast;
use std::collections::HashSet;

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
fn test_collect_imports_from_program_cache_hit() {
    let source = "import { A } from './file_to_import';";
    let temp_dir = tempfile::tempdir().unwrap();
    let pwd = temp_dir.path();
    let path = pwd.join("dummy.ts");
    let mut import_cache = std::collections::HashMap::new();

    let cached_imports = vec![PathBuf::from("/mock/cached/path.ts")];
    import_cache.insert(path.clone(), cached_imports.clone());

    ast::with_program(&path, source, |program, _| {
        let imports = collect_imports_from_program(&path, program, &mut import_cache);
        assert_eq!(imports, cached_imports);
    })
    .unwrap();
}

#[test]
fn test_collect_imports_from_program() {
    let source = r#"
        import { A } from './normal_import';
        export { B } from './named_export';
        export * from './export_all';
        import type { C } from './type_import';
        export type { D } from './type_export';
        export type * from './type_export_all';
    "#;

    let temp_dir = tempfile::tempdir().unwrap();
    let pwd = temp_dir.path();
    let path = pwd.join("dummy.ts");

    // Create dummy files so resolve_import returns Some
    std::fs::write(pwd.join("normal_import.ts"), "").unwrap();
    std::fs::write(pwd.join("named_export.ts"), "").unwrap();
    std::fs::write(pwd.join("export_all.ts"), "").unwrap();

    let mut import_cache = std::collections::HashMap::new();

    ast::with_program(&path, source, |program, _| {
        let imports = collect_imports_from_program(&path, program, &mut import_cache);

        assert_eq!(imports.len(), 3);
        assert!(imports.iter().any(|p| p.to_string_lossy().contains("normal_import")));
        assert!(imports.iter().any(|p| p.to_string_lossy().contains("named_export")));
        assert!(imports.iter().any(|p| p.to_string_lossy().contains("export_all")));

        assert_eq!(import_cache.get(&path).unwrap(), &imports);
    })
    .unwrap();
}
