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
    resolve(facts, resolver, remapper, path, local, want, &mut visited)
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

fn resolve<R: ImportResolverFacade>(
    facts: &std::collections::HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
    path: &Path,
    local: &str,
    want: Want,
    visited: &mut HashSet<PathBuf>,
) -> Option<String> {
    let (specifier, imported) = facts.get(path)?.string_imports.get(local)?.clone();
    let resolved = resolver.resolve(&specifier, path)?;
    let resolved = remapper.remap(&resolved)?;
    if !visited.insert(resolved.clone()) {
        return None;
    }
    export_named(
        facts, resolver, remapper, &resolved, &imported, want, visited,
    )
}

fn export_named<R: ImportResolverFacade>(
    facts: &std::collections::HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
    path: &Path,
    imported: &str,
    want: Want,
    visited: &mut HashSet<PathBuf>,
) -> Option<String> {
    let other = facts.get(path)?;
    if let Some(value) = direct_export(other, imported, want) {
        return Some(value);
    }
    if other.string_imports.contains_key(imported) {
        if let Some(value) = resolve(facts, resolver, remapper, path, imported, want, visited) {
            return Some(value);
        }
    }
    for specifier in &other.star_exports {
        let Some(next) = resolver
            .resolve(specifier, path)
            .and_then(|next| remapper.remap(&next))
        else {
            continue;
        };
        if !visited.insert(next.clone()) {
            continue;
        }
        if let Some(value) = export_named(facts, resolver, remapper, &next, imported, want, visited)
        {
            return Some(value);
        }
    }
    None
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
