pub fn collect_imports_from_program<'a>(
    abs_path: &Path,
    program: &oxc_ast::ast::Program<'a>,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
) -> Vec<PathBuf> {
    collect_imports_from_program_inner(abs_path, program, import_cache, None)
}

pub(crate) fn collect_imports_from_program_from_visible<'a>(
    abs_path: &Path,
    program: &oxc_ast::ast::Program<'a>,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
    visible_files: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    collect_imports_from_program_inner(abs_path, program, import_cache, Some(visible_files))
}

fn collect_imports_from_program_inner<'a>(
    abs_path: &Path,
    program: &oxc_ast::ast::Program<'a>,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Vec<PathBuf> {
    if let Some(cached_imports) = import_cache.get(abs_path) {
        return cached_imports.clone();
    }
    let mut imports = Vec::new();
    for stmt in &program.body {
        match stmt {
            Statement::ImportDeclaration(import) if is_runtime_import(import) => {
                if let Some(resolved) = resolve_import_with_visibility(
                    abs_path, import.source.value.as_str(), visible_files,
                ) {
                    imports.push(resolved);
                }
            }
            Statement::ExportNamedDeclaration(export) if is_runtime_export(export) => {
                if let Some(source) = &export.source {
                    if let Some(resolved) = resolve_import_with_visibility(
                        abs_path, source.value.as_str(), visible_files,
                    ) {
                        imports.push(resolved);
                    }
                }
            }
            Statement::ExportAllDeclaration(export)
                if export.export_kind != ImportOrExportKind::Type =>
            {
                if let Some(resolved) = resolve_import_with_visibility(
                    abs_path, export.source.value.as_str(), visible_files,
                ) {
                    imports.push(resolved);
                }
            }
            _ => {}
        }
    }
    import_cache.insert(abs_path.to_path_buf(), imports.clone());
    imports
}

fn resolve_import_with_visibility(
    abs_path: &Path,
    specifier: &str,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Option<PathBuf> {
    match visible_files {
        Some(visible) => resolve_import_from_visible(abs_path, specifier, visible),
        None => resolve_import(abs_path, specifier),
    }
}
