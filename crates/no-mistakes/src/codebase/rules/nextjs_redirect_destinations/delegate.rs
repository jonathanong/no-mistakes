use super::scan::{check_named_section, contains_word};
use super::RULE_ID;
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path};
use oxc_ast::ast::{
    ExportDefaultDeclarationKind, ExportFromDeclaration, ImportDeclarationSpecifier,
    ModuleExportName, Program, Statement,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const EXTENSIONS: &[&str] = &["ts", "mts", "cts", "tsx", "js", "mjs", "cjs", "jsx"];

pub(super) struct DelegateRequest<'a> {
    pub(super) root: &'a Path,
    pub(super) config_path: &'a Path,
    pub(super) source: &'a str,
    pub(super) files: &'a [PathBuf],
    pub(super) sources: &'a crate::codebase::ts_source::SourceStore,
    pub(super) route_set: &'a BTreeSet<String>,
    pub(super) app_root_label: &'a str,
    pub(super) include_rewrites: bool,
}

pub(super) fn append_delegated(
    request: &DelegateRequest<'_>,
    findings: &mut Vec<super::super::RuleFinding>,
) {
    let follow_redirects = !contains_word(request.source, "redirects");
    let follow_rewrites = request.include_rewrites && !contains_word(request.source, "rewrites");
    if !follow_redirects && !follow_rewrites {
        return;
    }
    let Some(delegated) = local_default_module(request.config_path, request.source, request.files)
    else {
        return;
    };
    let Some(delegated_source) = super::super::read_source(request.sources, &delegated) else {
        return;
    };
    if has_disable_file_comment(&delegated_source, RULE_ID) {
        return;
    }
    let rel = relative_slash_path(request.root, &delegated);
    if follow_redirects {
        findings.extend(check_named_section(
            &rel,
            &delegated_source,
            &delegated,
            "redirects",
            "redirect",
            request.route_set,
            request.app_root_label,
        ));
    }
    if follow_rewrites {
        findings.extend(check_named_section(
            &rel,
            &delegated_source,
            &delegated,
            "rewrites",
            "rewrite",
            request.route_set,
            request.app_root_label,
        ));
    }
}

fn local_default_module(config_path: &Path, source: &str, files: &[PathBuf]) -> Option<PathBuf> {
    let specifier = crate::ast::with_program(config_path, source, |program, _| {
        default_module_specifier(program).map(str::to_string)
    })
    .ok()
    .flatten()?;
    resolve_relative(config_path, &specifier, files)
}

fn default_module_specifier<'a>(program: &'a Program<'a>) -> Option<&'a str> {
    let mut imported: Option<(&str, &str)> = None;
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
                        imported =
                            Some((specifier.local.name.as_str(), import.source.value.as_str()));
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
    match (imported, exported_ident) {
        (Some((local, specifier)), Some(exported)) if local == exported => Some(specifier),
        _ => None,
    }
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

fn resolve_relative(from: &Path, specifier: &str, files: &[PathBuf]) -> Option<PathBuf> {
    if !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return None;
    }
    let base = from.parent()?.join(specifier);
    let normalized = crate::codebase::ts_resolver::normalize_path(&base);
    let mut candidates = vec![normalized.clone()];
    if normalized.extension().is_none() {
        for extension in EXTENSIONS {
            candidates.push(normalized.with_extension(*extension));
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
