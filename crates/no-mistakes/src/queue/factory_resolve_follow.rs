use super::{FileFacts, ImportResolverFacade};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub(in crate::queue) enum Want<'a> {
    Property(&'a str),
    Queue,
    Either,
    Definition,
}

pub(in crate::queue) fn lookup<R: ImportResolverFacade>(
    facts: &std::collections::HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
    path: &Path,
    local: &str,
    want: Want<'_>,
) -> Option<String> {
    Follow::new(facts, resolver, remapper).resolve(path, local, want)
}

pub(super) fn reexport_queue<R: ImportResolverFacade>(
    facts: &std::collections::HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
    path: &Path,
    file: &FileFacts,
    exported: &str,
    local: &str,
) -> Option<String> {
    if let Some((specifier, imported)) = file.export_from.get(exported).cloned() {
        return lookup_export(
            facts,
            resolver,
            remapper,
            path,
            &specifier,
            &imported,
            Want::Queue,
        );
    }
    file.queue_bindings
        .get(local)
        .cloned()
        .or_else(|| lookup(facts, resolver, remapper, path, local, Want::Queue))
}

pub(in crate::queue) fn lookup_export<R: ImportResolverFacade>(
    facts: &std::collections::HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
    path: &Path,
    specifier: &str,
    imported: &str,
    want: Want<'_>,
) -> Option<String> {
    Follow::new(facts, resolver, remapper).follow_source(path, specifier, imported, want)
}

pub(in crate::queue) fn lookup_queue_origin<R: ImportResolverFacade>(
    facts: &std::collections::HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
    path: &Path,
    local: &str,
) -> Option<(PathBuf, String)> {
    let mut follow = Follow::new(facts, resolver, remapper);
    let value = follow.resolve(path, local, Want::Definition)?;
    Some((follow.origin?, value))
}

struct Follow<'a, R: ImportResolverFacade> {
    facts: &'a std::collections::HashMap<PathBuf, FileFacts>,
    resolver: &'a R,
    remapper: &'a crate::codebase::ts_source::FrozenPathRemapper,
    visited: HashSet<(PathBuf, String)>,
    origin: Option<PathBuf>,
}

impl<'a, R: ImportResolverFacade> Follow<'a, R> {
    fn new(
        facts: &'a std::collections::HashMap<PathBuf, FileFacts>,
        resolver: &'a R,
        remapper: &'a crate::codebase::ts_source::FrozenPathRemapper,
    ) -> Self {
        Self {
            facts,
            resolver,
            remapper,
            visited: HashSet::new(),
            origin: None,
        }
    }

    fn resolve(&mut self, path: &Path, local: &str, want: Want<'_>) -> Option<String> {
        let file = self.facts.get(path)?;
        // A local import wins over an `export { ... as local } from` of the same name.
        let (specifier, imported) = file
            .string_imports
            .get(local)
            .or_else(|| file.export_from.get(local))?
            .clone();
        self.follow_source(path, &specifier, &imported, want)
    }

    fn follow_source(
        &mut self,
        path: &Path,
        specifier: &str,
        imported: &str,
        want: Want<'_>,
    ) -> Option<String> {
        let resolved = self.resolver.resolve(specifier, path)?;
        let resolved = self.remapper.remap(&resolved)?;
        if !self
            .visited
            .insert((resolved.clone(), imported.to_string()))
        {
            return None;
        }
        self.export_named(&resolved, imported, want)
    }

    fn export_named(&mut self, path: &Path, imported: &str, want: Want<'_>) -> Option<String> {
        let other = self.facts.get(path)?;
        let direct = match want {
            Want::Definition => self.local_definition(path, other, imported),
            _ => direct_export(other, imported, want),
        };
        if direct.is_some()
            || matches!(want, Want::Property(_))
                && other.exported_enqueue_properties.contains_key(imported)
        {
            return direct;
        }
        if let Some((specifier, name)) = other.export_from.get(imported).cloned() {
            if let Some(value) = self.follow_source(path, &specifier, &name, want) {
                return Some(value);
            }
        }
        if let Some(value) = self.resolve(path, export::imported_local(other, imported, want), want)
        {
            return Some(value);
        }
        for specifier in &other.star_exports {
            let Some(next) = self
                .resolver
                .resolve(specifier, path)
                .and_then(|next| self.remapper.remap(&next))
            else {
                continue;
            };
            if !self.visited.insert((next.clone(), imported.to_string())) {
                continue;
            }
            if let Some(value) = self.export_named(&next, imported, want) {
                return Some(value);
            }
        }
        None
    }

    fn local_definition(&mut self, path: &Path, file: &FileFacts, name: &str) -> Option<String> {
        if file.export_from.contains_key(name) {
            return None;
        }
        file.queue_exports.get(name)?;
        let local = file
            .reexports
            .iter()
            .find(|(exported, _)| exported == name)
            .map_or(name, |(_, local)| local);
        if let Some(queue_name) = file.queue_bindings.get(local) {
            self.origin = Some(path.to_path_buf());
            return Some(queue_name.clone());
        }
        if local != name && self.visited.insert((path.to_path_buf(), local.to_string())) {
            return self
                .resolve(path, local, Want::Definition)
                .or_else(|| self.export_named(path, local, Want::Definition));
        }
        None
    }
}

#[path = "factory_resolve_export.rs"]
mod export;
use export::direct_export;
