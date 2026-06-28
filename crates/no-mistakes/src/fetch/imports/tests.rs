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
fn test_is_import_used() {
    let source = r#"
        import "side_effect_import";
        import default_import from "default";
        import * as namespace_import from "namespace";
        import { named_import } from "named";
        import { unused_import } from "unused";
        import {} from "empty";
    "#;

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("dummy.ts");

    ast::with_program(&path, source, |program, _| {
        let mut referenced_identifiers = HashSet::new();
        referenced_identifiers.insert("default_import".to_string());
        referenced_identifiers.insert("namespace_import".to_string());
        referenced_identifiers.insert("named_import".to_string());

        let mut imports = program.body.iter().filter_map(|stmt| {
            if let oxc_ast::ast::Statement::ImportDeclaration(import) = stmt {
                Some(import)
            } else {
                None
            }
        });

        // 1. side_effect_import
        assert!(is_import_used(imports.next().unwrap(), &referenced_identifiers));

        // 2. default_import
        assert!(is_import_used(imports.next().unwrap(), &referenced_identifiers));

        // 3. namespace_import
        assert!(is_import_used(imports.next().unwrap(), &referenced_identifiers));

        // 4. named_import
        assert!(is_import_used(imports.next().unwrap(), &referenced_identifiers));

        // 5. unused_import
        assert!(!is_import_used(imports.next().unwrap(), &referenced_identifiers));

        // 6. empty import
        assert!(is_import_used(imports.next().unwrap(), &referenced_identifiers));
    })
    .unwrap();
}
