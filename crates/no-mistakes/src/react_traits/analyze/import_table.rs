use crate::import_shape::is_runtime_import;
use crate::imports::resolve_import;
use oxc_ast::ast::{ImportDeclarationSpecifier, Program};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct ImportEntry {
    pub(crate) resolved_path: PathBuf,
    pub(crate) exported_name: String,
}

pub(crate) type ImportTable = HashMap<String, ImportEntry>;

#[cfg(test)]
mod tests;

pub(crate) fn build_import_table(abs_path: &Path, program: &Program<'_>) -> ImportTable {
    build_import_table_inner(abs_path, program, None)
}

pub(crate) fn build_import_table_from_visible(
    abs_path: &Path,
    program: &Program<'_>,
    visible_files: &HashSet<PathBuf>,
) -> ImportTable {
    build_import_table_inner(abs_path, program, Some(visible_files))
}

fn build_import_table_inner(
    abs_path: &Path,
    program: &Program<'_>,
    visible_files: Option<&HashSet<PathBuf>>,
) -> ImportTable {
    let mut table = ImportTable::new();
    for stmt in &program.body {
        let oxc_ast::ast::Statement::ImportDeclaration(import) = stmt else {
            continue;
        };
        if !is_runtime_import(import) {
            continue;
        }
        let resolved = match visible_files {
            Some(visible) => crate::fetch::resolve::resolve_import_from_visible(
                abs_path,
                import.source.value.as_str(),
                visible,
            ),
            None => resolve_import(abs_path, import.source.value.as_str()),
        };
        let Some(resolved) = resolved else {
            continue;
        };
        let Some(specifiers) = &import.specifiers else {
            continue;
        };
        for specifier in specifiers {
            match specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                    table.insert(
                        s.local.name.to_string(),
                        ImportEntry {
                            resolved_path: resolved.clone(),
                            exported_name: "default".to_string(),
                        },
                    );
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                    table.insert(
                        s.local.name.to_string(),
                        ImportEntry {
                            resolved_path: resolved.clone(),
                            exported_name: "*".to_string(),
                        },
                    );
                }
                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                    table.insert(
                        s.local.name.to_string(),
                        ImportEntry {
                            resolved_path: resolved.clone(),
                            exported_name: s.imported.name().to_string(),
                        },
                    );
                }
            }
        }
    }
    table
}
