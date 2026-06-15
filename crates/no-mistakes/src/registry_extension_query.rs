//! `registry-extension <registry-file>`: summarize the repeated "register an
//! entry" shape used in a registry file so an agent can add a new entry that
//! follows the same pattern.
//!
//! This is intentionally heuristic. Three detectors run over the file and the
//! dominant one (by entry count) is reported:
//!
//! 1. `register-call` — a callee invoked >= 2 times whose argument references an
//!    imported symbol (e.g. `registry.register(new Foo())`).
//! 2. `container-array` / `container-object` — a default-exported array/object
//!    literal whose elements are entries.
//!
//! Side-effect imports (`import "./plugins/foo"`) are reported as a secondary
//! note. Mixed-shape files report the dominant shape and mention the rest in
//! `notes`; nothing is silently dropped.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use oxc::ast::ast::{
    Argument, CallExpression, ExportDefaultDeclarationKind, Expression, ImportDeclaration,
    ImportDeclarationSpecifier, ImportExpression, NewExpression,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
use serde::Serialize;

use crate::codebase::ts_source::{byte_offset_to_line, relative_slash_path};

/// The import backing a registry entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EntryImport {
    pub specifier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    pub local: String,
    pub kind: String,
}

/// One existing registry entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryEntry {
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none", rename = "import")]
    pub entry_import: Option<EntryImport>,
    pub call_shape: String,
}

/// The full `registry-extension` report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryExtensionReport {
    pub registry_file: String,
    pub pattern_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registrant: Option<String>,
    pub confidence: String,
    pub entries: Vec<RegistryEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    pub notes: Vec<String>,
}

/// Run the `registry-extension` query against a single file.
pub fn run(root: &Path, registry_file: &Path) -> Result<RegistryExtensionReport> {
    let path = if registry_file.is_absolute() {
        registry_file.to_path_buf()
    } else {
        root.join(registry_file)
    };
    let source = std::fs::read_to_string(&path)
        .map_err(|error| anyhow::anyhow!("cannot read {}: {error}", path.display()))?;
    let rel = relative_slash_path(root, &path);

    crate::ast::with_program(&path, &source, |program, source| {
        analyze(program, source, rel)
    })
    .map_err(|error| anyhow::anyhow!("{error}"))?
}

fn analyze(
    program: &oxc::ast::ast::Program<'_>,
    source: &str,
    registry_file: String,
) -> Result<RegistryExtensionReport> {
    let mut imports = ImportCollector {
        imports: HashMap::new(),
        side_effects: Vec::new(),
    };
    imports.visit_program(program);

    let mut body = BodyCollector {
        source,
        imports: &imports.imports,
        calls: Vec::new(),
        container: None,
    };
    body.visit_program(program);

    let mut notes = Vec::new();
    if !imports.side_effects.is_empty() {
        notes.push(format!(
            "also detected {} side-effect import registrant(s): {}",
            imports.side_effects.len(),
            imports.side_effects.join(", ")
        ));
    }

    // Group register-call candidates by callee key.
    let mut by_callee: HashMap<String, Vec<RawCall>> = HashMap::new();
    for call in body.calls {
        by_callee.entry(call.key.clone()).or_default().push(call);
    }
    // Pick the most-used registrant; break ties by callee key so the result is
    // deterministic across runs (HashMap iteration order is randomized).
    let register_best = by_callee
        .into_iter()
        .filter(|(_, calls)| calls.len() >= 2 && calls.iter().any(|c| c.entry_import.is_some()))
        .max_by(|a, b| a.1.len().cmp(&b.1.len()).then_with(|| b.0.cmp(&a.0)));

    let container = body.container.filter(|c| c.entries.len() >= 2);

    let register_count = register_best.as_ref().map_or(0, |(_, calls)| calls.len());
    let container_count = container.as_ref().map_or(0, |c| c.entries.len());

    if register_count == 0 && container_count == 0 {
        return Ok(RegistryExtensionReport {
            registry_file,
            pattern_kind: "none".to_string(),
            registrant: None,
            confidence: "low".to_string(),
            entries: Vec::new(),
            template: None,
            notes: push_default_note(notes),
        });
    }

    // Prefer the detector with more entries; tie -> register-call.
    if register_count >= container_count {
        let (key, mut calls) = register_best.expect("register_count > 0 implies a best callee");
        calls.sort_by_key(|c| c.line);
        if container_count > 0 {
            notes.push(format!(
                "also detected a container literal with {container_count} entries"
            ));
        }
        let registrant = calls
            .first()
            .map(|c| c.registrant.clone())
            .unwrap_or(key.clone());
        let entries: Vec<RegistryEntry> = calls
            .iter()
            .map(|c| RegistryEntry {
                line: c.line,
                entry_import: c.entry_import.clone(),
                call_shape: c.call_shape.clone(),
            })
            .collect();
        let template = entries.first().map(template_for_entry);
        let confidence = confidence_for(&entries);
        Ok(RegistryExtensionReport {
            registry_file,
            pattern_kind: "register-call".to_string(),
            registrant: Some(registrant),
            confidence,
            entries,
            template,
            notes,
        })
    } else {
        let container = container.expect("container_count > 0 implies a container");
        if register_count > 0 {
            notes.push(format!(
                "also detected a register-call shape with {register_count} entries"
            ));
        }
        let template = container.entries.first().map(template_for_entry);
        let confidence = confidence_for(&container.entries);
        Ok(RegistryExtensionReport {
            registry_file,
            pattern_kind: container.kind,
            registrant: None,
            confidence,
            entries: container.entries,
            template,
            notes,
        })
    }
}

fn push_default_note(mut notes: Vec<String>) -> Vec<String> {
    notes.push("no repeated registrant pattern detected (need >= 2 entries)".to_string());
    notes
}

fn confidence_for(entries: &[RegistryEntry]) -> String {
    if entries.iter().all(|e| e.entry_import.is_some()) {
        "high".to_string()
    } else {
        "medium".to_string()
    }
}

fn template_for_entry(entry: &RegistryEntry) -> String {
    match &entry.entry_import {
        // Replace the local identifier that actually appears in `call_shape`
        // (the imported `symbol` may be `default`/an alias). Dynamic-import
        // entries have no local name, so leave the shape verbatim.
        Some(import) if !import.local.is_empty() => {
            entry.call_shape.replacen(&import.local, "<Entry>", 1)
        }
        _ => entry.call_shape.clone(),
    }
}

include!("registry_extension_query/detect.rs");
include!("registry_extension_query/detect_resolve.rs");

#[cfg(test)]
mod tests;
