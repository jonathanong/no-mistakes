use crate::ast;
use crate::fetch::import_shape::{is_runtime_export, is_runtime_import};
use crate::fetch::resolve::resolve_import;
use anyhow::Result;
use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Statement};
use oxc_ast_visit::{walk, Visit};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::fetch::resolve::resolve_import_from_visible;

pub fn collect_imports(
    path: &Path,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
) -> Result<Vec<PathBuf>> {
    collect_imports_inner(path, import_cache, None)
}

pub(crate) fn collect_imports_from_visible(
    path: &Path,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
    visible_files: &HashSet<PathBuf>,
) -> Result<Vec<PathBuf>> {
    collect_imports_inner(path, import_cache, Some(visible_files))
}

fn collect_imports_inner(
    path: &Path,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Result<Vec<PathBuf>> {
    let abs_path = match visible_files {
        Some(_) => crate::codebase::ts_resolver::normalize_path(path),
        None => path.canonicalize()?,
    };
    if let Some(cached_imports) = import_cache.get(&abs_path) {
        return Ok(cached_imports.clone());
    }

    let source = std::fs::read_to_string(&abs_path)?;
    let imports = ast::with_program(path, &source, |program, _source| {
        collect_imports_from_program_inner(&abs_path, program, import_cache, visible_files)
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
    collect_runtime_imports_from_program_inner(abs_path, program, referenced_identifiers, None)
}

pub(crate) fn collect_runtime_imports_from_program_from_visible<'a>(
    abs_path: &Path,
    program: &oxc_ast::ast::Program<'a>,
    referenced_identifiers: &HashSet<String>,
    visible_files: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    collect_runtime_imports_from_program_inner(
        abs_path,
        program,
        referenced_identifiers,
        Some(visible_files),
    )
}

fn collect_runtime_imports_from_program_inner<'a>(
    abs_path: &Path,
    program: &oxc_ast::ast::Program<'a>,
    referenced_identifiers: &HashSet<String>,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Vec<PathBuf> {
    let mut imports = Vec::new();
    for stmt in &program.body {
        if let Statement::ImportDeclaration(import) = stmt {
            if !is_runtime_import(import) || !is_import_used(import, referenced_identifiers) {
                continue;
            }
            let resolved = match visible_files {
                Some(visible) => {
                    resolve_import_from_visible(abs_path, import.source.value.as_str(), visible)
                }
                None => resolve_import(abs_path, import.source.value.as_str()),
            };
            if let Some(resolved) = resolved {
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

include!("imports/program.rs");

#[cfg(test)]
mod tests;
