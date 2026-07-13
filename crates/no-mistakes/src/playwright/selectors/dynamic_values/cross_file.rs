use super::collect::{collect_object_string_values, collect_string_leaves};
use oxc_ast::ast::{
    Declaration, ExportDefaultDeclarationKind, Expression, ImportDeclarationSpecifier,
    ObjectPropertyKind, Program, Statement,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const DEFERRED_IMPORT_PREFIX: &str = "\0no-mistakes-playwright-import:";

#[derive(Clone, Default)]
pub(crate) struct StaticExportValues {
    named: std::collections::HashMap<String, Vec<String>>,
    default: Vec<String>,
}

impl StaticExportValues {
    pub(crate) fn values(&self, exported_name: &str, is_default: bool) -> &[String] {
        if is_default {
            &self.default
        } else {
            self.named
                .get(exported_name)
                .map(Vec::as_slice)
                .unwrap_or_default()
        }
    }
}

pub(crate) fn resolve_imported_values(
    local_name: &str,
    program: &Program<'_>,
    importing_file: &Path,
) -> Vec<String> {
    resolve_imported_values_inner(local_name, program, importing_file, None)
}

pub(crate) fn resolve_imported_values_from_visible(
    local_name: &str,
    program: &Program<'_>,
    importing_file: &Path,
    visible_files: &HashSet<PathBuf>,
) -> Vec<String> {
    resolve_imported_values_inner(local_name, program, importing_file, Some(visible_files))
}

pub(crate) fn defer_imported_values_from_visible(
    local_name: &str,
    program: &Program<'_>,
    importing_file: &Path,
    visible_files: &HashSet<PathBuf>,
) -> Vec<String> {
    let Some((source, exported_name, is_default)) = find_import_info(local_name, program) else {
        return Vec::new();
    };
    let Some(path) =
        crate::fetch::resolve::resolve_import_from_visible(importing_file, &source, visible_files)
    else {
        return Vec::new();
    };
    let payload = serde_json::to_string(&(
        crate::codebase::ts_resolver::normalize_path(&path),
        exported_name,
        is_default,
    ))
    .expect("deferred Playwright import marker is serializable");
    vec![format!("{DEFERRED_IMPORT_PREFIX}{payload}")]
}

pub(crate) fn resolve_deferred_import<'a>(
    value: &str,
    exports: &'a std::collections::HashMap<PathBuf, StaticExportValues>,
) -> Option<&'a [String]> {
    let payload = value.strip_prefix(DEFERRED_IMPORT_PREFIX)?;
    let (path, exported_name, is_default): (PathBuf, String, bool) =
        serde_json::from_str(payload).ok()?;
    Some(
        exports
            .get(&path)
            .map(|values| values.values(&exported_name, is_default))
            .unwrap_or_default(),
    )
}

pub(crate) fn collect_static_export_values(program: &Program<'_>) -> StaticExportValues {
    let mut facts = StaticExportValues::default();
    for statement in &program.body {
        match statement {
            Statement::ExportNamedDeclaration(export) => {
                let Some(declaration) = &export.declaration else {
                    continue;
                };
                match declaration {
                    Declaration::VariableDeclaration(variable) => {
                        for declarator in &variable.declarations {
                            let Some(name) = binding_ident_name(&declarator.id) else {
                                continue;
                            };
                            let mut values = Vec::new();
                            collect_from_named_declaration(declaration, &name, &mut values);
                            if !values.is_empty() {
                                facts.named.insert(name, values);
                            }
                        }
                    }
                    Declaration::FunctionDeclaration(function) => {
                        let Some(name) = function.id.as_ref().map(|id| id.name.to_string()) else {
                            continue;
                        };
                        let mut values = Vec::new();
                        collect_from_named_declaration(declaration, &name, &mut values);
                        if !values.is_empty() {
                            facts.named.insert(name, values);
                        }
                    }
                    _ => {}
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                collect_from_default_export(&export.declaration, &mut facts.default);
            }
            _ => {}
        }
    }
    facts
}

fn resolve_imported_values_inner(
    local_name: &str,
    program: &Program<'_>,
    importing_file: &Path,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Vec<String> {
    let Some((source_str, exported_name, is_default)) = find_import_info(local_name, program)
    else {
        return vec![];
    };

    let resolved_path = match visible_files {
        Some(visible) => {
            crate::fetch::resolve::resolve_import_from_visible(importing_file, &source_str, visible)
        }
        None => crate::fetch::resolve::resolve_import(importing_file, &source_str),
    };
    let Some(resolved_path) = resolved_path else {
        return vec![];
    };

    let Ok(source) = std::fs::read_to_string(&resolved_path) else {
        return vec![];
    };

    crate::playwright::ast::with_program(&resolved_path, &source, |target_program, _| {
        collect_exported_values(target_program, &exported_name, is_default)
    })
    .unwrap_or_default()
}

fn find_import_info(local_name: &str, program: &Program<'_>) -> Option<(String, String, bool)> {
    program.body.iter().find_map(|stmt| {
        let Statement::ImportDeclaration(import) = stmt else {
            return None;
        };

        import
            .specifiers
            .as_ref()?
            .iter()
            .find_map(|specifier| match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(named)
                    if named.local.name == local_name =>
                {
                    Some((
                        import.source.value.to_string(),
                        named.imported.name().to_string(),
                        false,
                    ))
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(default)
                    if default.local.name == local_name =>
                {
                    Some((import.source.value.to_string(), "default".to_string(), true))
                }
                _ => None,
            })
    })
}

fn collect_exported_values(
    program: &Program<'_>,
    exported_name: &str,
    is_default: bool,
) -> Vec<String> {
    let mut values = Vec::new();

    for stmt in &program.body {
        match stmt {
            Statement::ExportNamedDeclaration(export) => {
                if let Some(decl) = &export.declaration {
                    collect_from_named_declaration(decl, exported_name, &mut values);
                }
            }
            Statement::ExportDefaultDeclaration(export) if is_default => {
                collect_from_default_export(&export.declaration, &mut values);
            }
            _ => {}
        }
    }

    values
}

include!("cross_file_collect.rs");
