use crate::ast;
use crate::fetch::import_shape::{is_runtime_export, is_runtime_import};
use crate::fetch::resolve::resolve_import;
use anyhow::Result;
use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Statement};
use oxc_ast_visit::{walk, Visit};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn collect_imports(
    path: &Path,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
) -> Result<Vec<PathBuf>> {
    let abs_path = path.canonicalize()?;
    if let Some(cached_imports) = import_cache.get(&abs_path) {
        return Ok(cached_imports.clone());
    }

    let source = std::fs::read_to_string(&abs_path)?;
    let imports = ast::with_program(path, &source, |program, _source| {
        collect_imports_from_program(&abs_path, program, import_cache)
    })?;
    Ok(imports)
}

#[derive(Default)]
pub struct IdentifierReferenceCollector {
    pub identifiers: HashSet<String>,
}

impl<'a> Visit<'a> for IdentifierReferenceCollector {
    fn visit_identifier_reference(&mut self, it: &oxc_ast::ast::IdentifierReference<'a>) {
        self.identifiers.insert(it.name.to_string());
        walk::walk_identifier_reference(self, it);
    }
}

pub fn collect_identifier_references(program: &oxc_ast::ast::Program<'_>) -> HashSet<String> {
    let mut collector = IdentifierReferenceCollector::default();
    collector.visit_program(program);
    collector.identifiers
}

pub fn collect_runtime_imports_from_program<'a>(
    abs_path: &Path,
    program: &oxc_ast::ast::Program<'a>,
    referenced_identifiers: &HashSet<String>,
) -> Vec<PathBuf> {
    let mut imports = Vec::new();
    for stmt in &program.body {
        if let Statement::ImportDeclaration(import) = stmt {
            if !is_runtime_import(import) || !is_import_used(import, referenced_identifiers) {
                continue;
            }
            if let Some(resolved) = resolve_import(abs_path, import.source.value.as_str()) {
                imports.push(resolved);
            }
        }
    }
    imports
}

pub fn is_import_used(
    import: &oxc_ast::ast::ImportDeclaration<'_>,
    referenced_identifiers: &HashSet<String>,
) -> bool {
    let Some(specifiers) = &import.specifiers else {
        return true;
    };
    if specifiers.is_empty() {
        return true;
    }

    for specifier in specifiers {
        let local_name = match specifier {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(default_import) => {
                default_import.local.name.as_ref()
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace_import) => {
                namespace_import.local.name.as_ref()
            }
            ImportDeclarationSpecifier::ImportSpecifier(import_specifier) => {
                import_specifier.local.name.as_ref()
            }
        };
        if referenced_identifiers.contains(local_name) {
            return true;
        }
    }

    false
}

pub fn collect_imports_from_program<'a>(
    abs_path: &Path,
    program: &oxc_ast::ast::Program<'a>,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
) -> Vec<PathBuf> {
    if let Some(cached_imports) = import_cache.get(abs_path) {
        return cached_imports.clone();
    }

    let mut imports = Vec::new();
    for stmt in &program.body {
        match stmt {
            Statement::ImportDeclaration(import) if is_runtime_import(import) => {
                if let Some(resolved) = resolve_import(abs_path, import.source.value.as_str()) {
                    imports.push(resolved);
                }
            }
            Statement::ExportNamedDeclaration(export) => {
                if !is_runtime_export(export) {
                    continue;
                }
                if let Some(source) = &export.source {
                    if let Some(resolved) = resolve_import(abs_path, source.value.as_str()) {
                        imports.push(resolved);
                    }
                }
            }
            Statement::ExportAllDeclaration(export) => {
                if export.export_kind == ImportOrExportKind::Type {
                    continue;
                }
                if let Some(resolved) = resolve_import(abs_path, export.source.value.as_str()) {
                    imports.push(resolved);
                }
            }
            _ => {}
        }
    }

    import_cache.insert(abs_path.to_path_buf(), imports.clone());
    imports
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast;
    use std::collections::HashSet;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_collect_runtime_imports_from_program() {
        let dir = tempdir().unwrap();
        let main_path = dir.path().join("main.ts");
        let util_path = dir.path().join("util.ts");
        let type_path = dir.path().join("types.ts");
        let unused_path = dir.path().join("unused.ts");

        // Set up test files
        fs::write(
            &main_path,
            "
            import { usedVar } from './util';
            import type { SomeType } from './types';
            import { unusedVar } from './unused';
            console.log(usedVar);
            ",
        )
        .unwrap();
        fs::write(&util_path, "export const usedVar = 1;").unwrap();
        fs::write(&type_path, "export type SomeType = string;").unwrap();
        fs::write(&unused_path, "export const unusedVar = 2;").unwrap();

        let source = fs::read_to_string(&main_path).unwrap();
        let mut referenced_identifiers = HashSet::new();
        referenced_identifiers.insert("usedVar".to_string());

        let imports = ast::with_program(&main_path, &source, |program, _| {
            collect_runtime_imports_from_program(&main_path, program, &referenced_identifiers)
        })
        .unwrap();

        assert_eq!(imports.len(), 1);

        let resolved_util_path = util_path.canonicalize().unwrap_or(util_path);
        let first_import = imports[0].canonicalize().unwrap_or(imports[0].clone());
        assert_eq!(first_import, resolved_util_path);
    }
}
