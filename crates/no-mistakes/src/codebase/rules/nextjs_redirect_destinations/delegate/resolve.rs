use oxc_ast::ast::{
    ExportDefaultDeclarationKind, ExportFromDeclaration, ImportDeclarationSpecifier,
    ModuleExportName, Program, Statement,
};
use std::path::{Path, PathBuf};
const EXTENSIONS: &[&str] = &["ts", "mts", "cts", "tsx", "js", "mjs", "cjs", "jsx"];
pub(super) fn default_module_specifier<'a>(program: &'a Program<'a>) -> Option<&'a str> {
    let mut imported = Vec::new();
    let mut exported_ident: Option<&str> = None;
    let mut reexport: Option<&str> = None;
    for statement in &program.body {
        match statement {
            Statement::ImportDeclaration(import) => {
                let Some(specifiers) = &import.specifiers else {
                    continue;
                };
                for specifier in specifiers {
                    if let ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) = specifier
                    {
                        imported
                            .push((specifier.local.name.as_str(), import.source.value.as_str()));
                    }
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                if let ExportDefaultDeclarationKind::Identifier(identifier) = &export.declaration {
                    exported_ident = Some(identifier.name.as_str());
                }
            }
            Statement::ExportFromDeclaration(export) => {
                if let Some(specifier) = default_reexport_specifier(export) {
                    reexport = Some(specifier);
                }
            }
            _ => {}
        }
    }
    if let Some(specifier) = reexport {
        return Some(specifier);
    }
    let exported = exported_ident?;
    imported
        .into_iter()
        .find(|(local, _)| *local == exported)
        .map(|(_, specifier)| specifier)
}

fn default_reexport_specifier<'a>(export: &'a ExportFromDeclaration<'a>) -> Option<&'a str> {
    let reexports_default = export.specifiers.iter().any(|specifier| {
        export_name(&specifier.local) == "default" && export_name(&specifier.exported) == "default"
    });
    reexports_default.then_some(export.source.value.as_str())
}

fn export_name<'a>(name: &'a ModuleExportName<'a>) -> &'a str {
    match name {
        ModuleExportName::IdentifierName(identifier) => identifier.name.as_str(),
        ModuleExportName::IdentifierReference(identifier) => identifier.name.as_str(),
        ModuleExportName::StringLiteral(literal) => literal.value.as_str(),
    }
}

pub(super) fn resolve_relative(from: &Path, specifier: &str, files: &[PathBuf]) -> Option<PathBuf> {
    if !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return None;
    }
    let base = from.parent()?.join(specifier);
    let normalized = crate::codebase::ts_resolver::normalize_path(&base);
    let mut candidates = Vec::new();
    match source_extension(&normalized) {
        Some(extension) => {
            candidates.push(normalized.clone());
            if let Some(source_extension) = typescript_extension(extension) {
                candidates.push(normalized.with_extension(source_extension));
            }
        }
        None => {
            for extension in EXTENSIONS {
                candidates.push(append_extension(&normalized, extension));
                candidates.push(normalized.join(format!("index.{extension}")));
            }
        }
    }
    files
        .iter()
        .find(|file| {
            let file = crate::codebase::ts_resolver::normalize_path(file);
            candidates.iter().any(|candidate| candidate == &file)
        })
        .cloned()
}

fn source_extension(path: &Path) -> Option<&str> {
    let extension = path.extension().and_then(|extension| extension.to_str())?;
    EXTENSIONS.contains(&extension).then_some(extension)
}

fn append_extension(path: &Path, extension: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".");
    name.push(extension);
    PathBuf::from(name)
}

fn typescript_extension(extension: &str) -> Option<&'static str> {
    const PAIRS: &[(&str, &str)] = &[("js", "ts"), ("jsx", "tsx"), ("mjs", "mts"), ("cjs", "cts")];
    PAIRS
        .iter()
        .find(|(javascript, _)| *javascript == extension)
        .map(|(_, typescript)| *typescript)
}
