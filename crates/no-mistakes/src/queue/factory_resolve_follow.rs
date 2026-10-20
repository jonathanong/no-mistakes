use super::{FileFacts, ImportResolverFacade, Update};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub(super) enum Want {
    Queue,
    Either,
}

pub(super) fn lookup<R: ImportResolverFacade>(
    facts: &std::collections::HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
    path: &Path,
    local: &str,
    want: Want,
) -> Option<String> {
    let mut visited = HashSet::new();
    Follow {
        facts,
        resolver,
        remapper,
        visited: &mut visited,
    }
    .resolve(path, local, want)
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

pub(super) fn lookup_export<R: ImportResolverFacade>(
    facts: &std::collections::HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
    path: &Path,
    specifier: &str,
    imported: &str,
    want: Want,
) -> Option<String> {
    let mut visited = HashSet::new();
    Follow {
        facts,
        resolver,
        remapper,
        visited: &mut visited,
    }
    .follow_source(path, specifier, imported, want)
}

pub(super) fn star_and_job_updates<R: ImportResolverFacade>(
    facts: &std::collections::HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> Vec<Update> {
    let mut updates = Vec::new();
    for (path, file) in facts {
        for specifier in &file.star_exports {
            let Some(resolved) = resolver
                .resolve(specifier, path)
                .and_then(|resolved| remapper.remap(&resolved))
            else {
                continue;
            };
            let other = &facts[&resolved];
            for (name, queue_name) in &other.queue_exports {
                if file.queue_exports.contains_key(name) {
                    continue;
                }
                updates.push(Update::Export {
                    file: path.clone(),
                    exported: name.clone(),
                    queue_name: queue_name.clone(),
                });
            }
        }
        for (index, producer) in file.producers.iter().enumerate() {
            if producer.job.is_some() {
                continue;
            }
            let Some(local) = &producer.job_import else {
                continue;
            };
            let Some(job) = lookup(facts, resolver, remapper, path, local, Want::Either) else {
                continue;
            };
            updates.push(Update::ProducerJob {
                file: path.clone(),
                index,
                job,
            });
        }
    }
    updates
}

struct Follow<'a, R: ImportResolverFacade> {
    facts: &'a std::collections::HashMap<PathBuf, FileFacts>,
    resolver: &'a R,
    remapper: &'a crate::codebase::ts_source::FrozenPathRemapper,
    visited: &'a mut HashSet<(PathBuf, String)>,
}

impl<R: ImportResolverFacade> Follow<'_, R> {
    fn resolve(&mut self, path: &Path, local: &str, want: Want) -> Option<String> {
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
        want: Want,
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

    fn export_named(&mut self, path: &Path, imported: &str, want: Want) -> Option<String> {
        let other = self.facts.get(path)?;
        if let Some(value) = direct_export(other, imported, want) {
            return Some(value);
        }
        if let Some((specifier, name)) = other.export_from.get(imported).cloned() {
            if let Some(value) = self.follow_source(path, &specifier, &name, want) {
                return Some(value);
            }
        }
        if other.string_imports.contains_key(imported) {
            if let Some(value) = self.resolve(path, imported, want) {
                return Some(value);
            }
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
}

fn direct_export(file: &FileFacts, imported: &str, want: Want) -> Option<String> {
    match want {
        Want::Queue => file.queue_exports.get(imported).cloned(),
        Want::Either => file
            .exported_strings
            .get(imported)
            .cloned()
            .or_else(|| file.queue_exports.get(imported).cloned()),
    }
}
