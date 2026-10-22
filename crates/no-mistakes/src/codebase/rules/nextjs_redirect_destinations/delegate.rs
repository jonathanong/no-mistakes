use super::extract::{extract_named_destinations_from_program, ExtractedDestinations};
use super::scan::section_findings;
use super::words::contains_word;
use super::RULE_ID;
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path};
use oxc_ast::ast::{
    ExportDefaultDeclarationKind, ExportFromDeclaration, ImportDeclarationSpecifier,
    ModuleExportName, Program, Statement,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const EXTENSIONS: &[&str] = &["ts", "mts", "cts", "tsx", "js", "mjs", "cjs", "jsx"];

#[derive(Default)]
pub(super) struct ConfigAnalysis {
    pub(super) redirects: ExtractedDestinations,
    pub(super) rewrites: ExtractedDestinations,
    pub(super) specifier: Option<String>,
}

pub(super) fn analyze_config(path: &Path, source: &str) -> ConfigAnalysis {
    crate::ast::with_program(path, source, |program, source| ConfigAnalysis {
        redirects: extract_named_destinations_from_program(program, source, "redirects"),
        rewrites: extract_named_destinations_from_program(program, source, "rewrites"),
        specifier: default_module_specifier(program).map(str::to_string),
    })
    .unwrap_or_default()
}

pub(super) struct DelegateRequest<'a> {
    pub(super) root: &'a Path,
    pub(super) config_path: &'a Path,
    pub(super) source: &'a str,
    pub(super) files: &'a [PathBuf],
    pub(super) sources: &'a crate::codebase::ts_source::SourceStore,
    pub(super) route_set: &'a BTreeSet<String>,
    pub(super) app_root_label: &'a str,
    pub(super) include_rewrites: bool,
    pub(super) specifier: Option<&'a str>,
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
    let Some(specifier) = request.specifier else {
        return;
    };
    let Some(delegated) = resolve_relative(request.config_path, specifier, request.files) else {
        return;
    };
    let Some(delegated_source) = super::super::read_source(request.sources, &delegated) else {
        return;
    };
    if has_disable_file_comment(&delegated_source, RULE_ID) {
        return;
    }
    let rel = relative_slash_path(request.root, &delegated);
    let analysis = analyze_config(&delegated, &delegated_source);
    if follow_redirects {
        findings.extend(section_findings(
            &rel,
            &delegated_source,
            "redirects",
            "redirect",
            &analysis.redirects,
            request.route_set,
            request.app_root_label,
        ));
    }
    if follow_rewrites {
        findings.extend(section_findings(
            &rel,
            &delegated_source,
            "rewrites",
            "rewrite",
            &analysis.rewrites,
            request.route_set,
            request.app_root_label,
        ));
    }
}

fn default_module_specifier<'a>(program: &'a Program<'a>) -> Option<&'a str> {
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

fn resolve_relative(from: &Path, specifier: &str, files: &[PathBuf]) -> Option<PathBuf> {
    if !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return None;
    }
    let base = from.parent()?.join(specifier);
    let normalized = crate::codebase::ts_resolver::normalize_path(&base);
    let mut candidates = Vec::new();
    match normalized
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some(extension) => {
            candidates.push(normalized.clone());
            if let Some(source_extension) = typescript_extension(extension) {
                candidates.push(normalized.with_extension(source_extension));
            }
        }
        None => {
            for extension in EXTENSIONS {
                candidates.push(normalized.with_extension(*extension));
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

fn typescript_extension(extension: &str) -> Option<&'static str> {
    match extension {
        "js" => Some("ts"),
        "jsx" => Some("tsx"),
        "mjs" => Some("mts"),
        "cjs" => Some("cts"),
        _ => None,
    }
}
