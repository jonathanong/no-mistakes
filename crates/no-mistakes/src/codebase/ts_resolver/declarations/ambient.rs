//! An index of declared module names, never a separate relationship graph.
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_resolver::TsConfigCatalog;
use crate::codebase::ts_source::facts::ancestry::ClassShape;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

type ClassExports = FxHashMap<String, Vec<(PathBuf, CallableId)>>;
type DeclaredModules = FxHashMap<String, ClassExports>;

pub(super) enum ClassLookup {
    Unique(PathBuf, CallableId),
    Ambiguous,
    Missing,
}

#[derive(Default)]
pub(super) struct AmbientModules {
    classes: FxHashMap<Option<usize>, DeclaredModules>,
}

impl AmbientModules {
    pub fn new(
        paths: &[PathBuf],
        facts: &dyn TsFactLookup,
        catalog: Option<&TsConfigCatalog>,
    ) -> Self {
        let mut result = Self::default();
        for path in paths {
            let Some(file) = facts.get_ts_facts(path) else {
                continue;
            };
            let owner = catalog.and_then(|catalog| catalog.resolver_scope_index_for(path));
            result.index_file(owner, path, file);
        }
        result
    }

    pub(super) fn index_file(&mut self, owner: Option<usize>, path: &Path, file: &TsFileFacts) {
        if file.parse_error.is_some() || file.operational_error.is_some() {
            return;
        }
        if file.ancestry.ambient_modules.is_empty()
            && file.ancestry.ambient_module_assignments.is_empty()
        {
            return;
        }
        let classes: FxHashMap<_, _> = file
            .ancestry
            .classes
            .iter()
            .map(|class| (class.id, class))
            .collect();
        for (name, ids) in &file.ancestry.ambient_modules {
            for class in ids.iter().filter_map(|id| classes.get(id)) {
                self.push(owner, path, name, &class.name, class.id);
            }
        }
        for (module, namespace) in &file.ancestry.ambient_module_assignments {
            for class in &file.ancestry.classes {
                if class.module.is_some() || !class.module_level {
                    continue;
                }
                let Some(exported) = assigned_export(class, namespace) else {
                    continue;
                };
                if namespace_exported(file, class) {
                    self.push(owner, path, module, &exported, class.id);
                }
            }
        }
    }

    fn push(
        &mut self,
        owner: Option<usize>,
        path: &Path,
        module: &str,
        exported: &str,
        id: CallableId,
    ) {
        let candidates = self
            .classes
            .entry(owner)
            .or_default()
            .entry(module.to_string())
            .or_default()
            .entry(exported.to_string())
            .or_default();
        if !candidates
            .iter()
            .any(|(existing, existing_id)| existing == path && *existing_id == id)
        {
            candidates.push((path.to_path_buf(), id));
        }
    }

    pub fn class(
        &self,
        specifier: &str,
        importer: &Path,
        exported: &str,
        catalog: Option<&TsConfigCatalog>,
    ) -> ClassLookup {
        let owner = catalog.and_then(|catalog| catalog.resolver_scope_index_for(importer));
        let Some(exports) = self
            .classes
            .get(&owner)
            .and_then(|modules| modules.get(specifier))
        else {
            return ClassLookup::Missing;
        };
        match exports.get(exported).map(Vec::as_slice) {
            Some([(path, id)]) => ClassLookup::Unique(path.clone(), *id),
            // Several classes for one export, or a module that already names
            // more than one class, is not proof. Do not consult another index.
            // An empty candidate list is never stored.
            Some(_) => ClassLookup::Ambiguous,
            None if module_is_ambiguous(exports) => ClassLookup::Ambiguous,
            None => ClassLookup::Missing,
        }
    }
}

fn module_is_ambiguous(exports: &ClassExports) -> bool {
    exports.len() > 1 || exports.values().any(|candidates| candidates.len() != 1)
}

fn assigned_export(class: &ClassShape, namespace: &str) -> Option<String> {
    let path = class.namespace.join(".");
    if path == namespace {
        return Some(class.name.clone());
    }
    let rest = path.strip_prefix(&format!("{namespace}."))?;
    Some(format!("{rest}.{}", class.name))
}

fn namespace_exported(file: &TsFileFacts, class: &ClassShape) -> bool {
    let bodies = &file.ancestry.namespace_bodies;
    let exported = bodies
        .iter()
        .filter(|body| body.namespace == class.namespace)
        .flat_map(|body| &body.values)
        .any(|value| value.class == Some(class.id) && value.exported);
    exported
        && (1..class.namespace.len()).all(|depth| {
            bodies
                .iter()
                .filter(|body| body.namespace == class.namespace[..depth])
                .flat_map(|body| &body.values)
                .any(|value| value.name == class.namespace[depth] && value.exported)
        })
}

#[cfg(test)]
#[path = "ambient_shapes_tests.rs"]
mod ambient_shapes_tests;
