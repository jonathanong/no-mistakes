//! Exact import/export projection over the request's prepared module facts.
use super::{
    lower::Lowered,
    values::{Catalog, Key},
};
use crate::codebase::{
    check_facts::CheckFileFacts,
    ts_source::{module_facts::TypeScriptModuleFacts, FileIdMap},
};
use std::path::{Path, PathBuf};

pub(super) struct PreparedCatalog<'a> {
    pub files: &'a FileIdMap<CheckFileFacts>,
    pub application: Option<&'a super::ApplicationKey>,
    pub profile: Option<&'a super::super::EmbeddedSqlOptions>,
    pub resolve: &'a dyn Fn(&str, &Path) -> Option<PathBuf>,
}
impl PreparedCatalog<'_> {
    fn module(&self, file: &Path) -> Option<&TypeScriptModuleFacts> {
        let facts = self.files.get(file)?;
        if self
            .application
            .is_some_and(|key| !facts.sql_identifier_syntax.includes_application(key))
        {
            return None;
        }
        if facts.parse_error.is_some()
            || facts.ts.parse_error.is_some()
            || facts.ts.operational_error.is_some()
        {
            return None;
        }
        let module = facts.ts.module_bindings.as_deref()?;
        module.diagnostics.is_empty().then_some(module)
    }
    pub fn binding(&self, file: &Path, id: usize) -> Option<Key> {
        self.resolve_binding(file, id, &mut Vec::new())
    }
    fn resolve_binding(
        &self,
        file: &Path,
        id: usize,
        visited: &mut Vec<(PathBuf, String)>,
    ) -> Option<Key> {
        let facts = self.files.get(file)?;
        let models = &facts.sql_identifier_syntax.recovery;
        if models.writes.contains(&id) {
            return None;
        }
        let module = self.module(file)?;
        if let Some(callable) = models.binding_functions.get(&id) {
            return callable.map(|callable| Key {
                file: file.to_path_buf(),
                callable,
            });
        }
        let mut imports = module
            .imports
            .iter()
            .flat_map(|import| import.bindings.iter().map(move |binding| (import, binding)))
            .filter(|(import, binding)| {
                !import.type_only
                    && !binding.type_only
                    && binding.binding_id == id
                    && binding.kind != "namespace"
            });
        let (import, binding) = imports.next()?;
        if imports.next().is_some() {
            return None;
        }
        let target = (self.resolve)(&import.specifier, file)?;
        self.export(&target, &binding.imported, visited)
    }
    fn export(&self, file: &Path, name: &str, visited: &mut Vec<(PathBuf, String)>) -> Option<Key> {
        let step = (file.to_path_buf(), name.to_string());
        if visited.len() >= 8 || visited.contains(&step) {
            return None;
        }
        visited.push(step);
        let module = self.module(file)?;
        let mut exports = module
            .exports
            .iter()
            .filter(|export| !export.type_only && export.exported == name);
        let export = exports.next()?;
        if exports.next().is_some() {
            return None;
        }
        if let Some(specifier) = &export.specifier {
            let target = (self.resolve)(specifier, file)?;
            self.export(&target, &export.local, visited)
        } else {
            let mut bindings = module.bindings.iter().filter(|binding| {
                binding.name == export.local && binding.scope_id == 0 && binding.runtime
            });
            let binding = bindings.next()?;
            if bindings.next().is_some() {
                return None;
            }
            self.resolve_binding(file, binding.id, visited)
        }
    }
}
impl Catalog for PreparedCatalog<'_> {
    fn function(&self, key: &Key) -> Option<&Lowered> {
        self.module(&key.file)?;
        self.files
            .get(&key.file)?
            .sql_identifier_syntax
            .recovery
            .functions
            .get(&key.callable)
            .map(|model| &model.lowered)
    }
    fn sink(&self, from: &Path, binding: usize) -> bool {
        self.profile
            .and_then(|profile| {
                self.files
                    .get(from)?
                    .sql_identifier_syntax
                    .recovery
                    .executors
                    .get(profile)
            })
            .is_some_and(|executors| executors.contains(&binding))
    }
    fn callee(&self, from: &Path, binding: usize) -> Option<Key> {
        self.binding(from, binding)
    }
}
