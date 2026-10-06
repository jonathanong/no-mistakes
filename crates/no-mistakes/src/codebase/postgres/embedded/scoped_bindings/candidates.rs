use super::super::relative::RelativeScopedCandidate;
use super::super::EmbeddedSqlOptions;
use oxc_ast::ast::{
    ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind, Program, Statement,
};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct ImportClassification {
    pub(super) factories: HashSet<String>,
    pub(super) types: HashSet<String>,
    pub(super) provisional_factories: HashMap<String, Vec<u32>>,
    pub(super) provisional_types: HashMap<String, Vec<u32>>,
    pub(super) candidates: Vec<RelativeScopedCandidate>,
}

pub(super) fn classify(
    program: &Program<'_>,
    options: &EmbeddedSqlOptions,
) -> ImportClassification {
    let mut classified = ImportClassification::default();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        classify_import(import, options, &mut classified);
    }
    classified
}

pub(super) fn from_configured_module(source: &str, specifier: &str) -> bool {
    specifier.is_empty()
        || source == specifier
        || source
            .strip_prefix(specifier)
            .is_some_and(|rest| rest.starts_with('/'))
}

pub(super) fn contains(names: &[String], name: &str) -> bool {
    names.iter().any(|candidate| candidate == name)
}

fn is_relative(source: &str) -> bool {
    source == "." || source == ".." || source.starts_with("./") || source.starts_with("../")
}

fn classify_import(
    import: &ImportDeclaration<'_>,
    options: &EmbeddedSqlOptions,
    classified: &mut ImportClassification,
) {
    let source = import.source.value.as_str();
    let confirmed = from_configured_module(source, &options.import_specifier);
    let relative = is_relative(source);
    let Some(specifiers) = &import.specifiers else {
        return;
    };
    for specifier in specifiers {
        let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
            continue;
        };
        let imported = named.imported.name();
        let local = named.local.name.to_string();
        let type_only = import.import_kind == ImportOrExportKind::Type
            || named.import_kind == ImportOrExportKind::Type;
        let factory = !type_only && contains(&options.executor_factory_names, imported.as_str());
        let type_import = contains(&options.executor_type_names, imported.as_str());
        if !factory && !type_import {
            continue;
        }
        if confirmed {
            if factory {
                classified.factories.insert(local.clone());
            }
            if type_import {
                classified.types.insert(local);
            }
            continue;
        }
        if options.import_specifier.is_empty() || !relative {
            continue;
        }
        let index = u32::try_from(classified.candidates.len()).unwrap_or(u32::MAX);
        if factory {
            classified
                .provisional_factories
                .entry(local.clone())
                .or_default()
                .push(index);
        }
        if type_import {
            classified
                .provisional_types
                .entry(local.clone())
                .or_default()
                .push(index);
        }
        classified.candidates.push(RelativeScopedCandidate {
            specifier: source.to_string(),
            imported_name: imported.to_string(),
            local_name: local,
            factory,
            type_import,
        });
    }
}
