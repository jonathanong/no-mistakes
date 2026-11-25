use super::{local_queues, producers::resolved_binding};
use crate::codebase::ts_resolver::ImportResolverFacade;
use crate::queue::extract::FileFacts;
use crate::queue::extract_model::{JobOptionSite, LocalQueue, ProducerBinding};
use crate::queue::types::QueueKey;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

struct ResolvedSite {
    queue_name: String,
    queue_file: Option<PathBuf>,
    declaration: Option<u32>,
    ambiguous_defaults: bool,
}

struct SiteResolveCtx<'a, R: ImportResolverFacade> {
    declaring_files: HashMap<&'a str, BTreeSet<&'a PathBuf>>,
    facts: &'a HashMap<PathBuf, FileFacts>,
    definitions: &'a HashMap<String, PathBuf>,
    resolver: &'a R,
    remapper: &'a crate::codebase::ts_source::FrozenPathRemapper,
}

pub(in crate::queue) fn resolve_job_option_sites<R: ImportResolverFacade>(
    facts: &mut HashMap<PathBuf, FileFacts>,
    queue_defs: &HashMap<PathBuf, HashMap<String, String>>,
    definitions: &HashMap<String, PathBuf>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) {
    let mut declaring_files = HashMap::<&str, BTreeSet<&PathBuf>>::new();
    for (path, file) in facts.iter() {
        for name in crate::queue::factory_resolve::defined_queue_names(file) {
            declaring_files.entry(name).or_default().insert(path);
        }
    }
    let ctx = SiteResolveCtx {
        declaring_files,
        facts,
        definitions,
        resolver,
        remapper,
    };
    let mut slots = Vec::new();
    for (path, file) in facts.iter() {
        let local = local_queues(path, file, queue_defs, definitions, resolver, remapper);
        for (index, site) in file.job_option_sites.iter().enumerate() {
            if let Some(resolved) = ctx.resolve_site(path, file, site, &local) {
                slots.push((path.clone(), index, resolved));
            }
        }
    }
    for (path, index, resolved) in slots {
        if let Some(file) = facts.get_mut(&path) {
            let site = &mut file.job_option_sites[index];
            site.queue_file = resolved.queue_file;
            site.queue_name = Some(resolved.queue_name);
            site.declaration = resolved.declaration;
            site.ambiguous_defaults = resolved.ambiguous_defaults;
        }
    }
}

impl<R: ImportResolverFacade> SiteResolveCtx<'_, R> {
    fn resolve_site(
        &self,
        path: &Path,
        file: &FileFacts,
        site: &JobOptionSite,
        local: &HashMap<String, QueueKey>,
    ) -> Option<ResolvedSite> {
        if let Some(ProducerBinding::Local(declaration)) = &site.binding {
            if let Some(LocalQueue::Known(name)) = file.queue_declarations.get(declaration) {
                return Some(ResolvedSite {
                    queue_name: name.clone(),
                    queue_file: Some(path.to_path_buf()),
                    declaration: Some(*declaration),
                    ambiguous_defaults: false,
                });
            }
        }
        let key = site
            .queue_name
            .as_ref()
            .map(|name| QueueKey {
                queue_file: self
                    .definitions
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| path.to_path_buf()),
                queue_name: name.clone(),
            })
            .or_else(|| {
                resolved_binding(
                    local,
                    self.facts,
                    self.resolver,
                    self.remapper,
                    path,
                    site.binding.as_ref()?,
                    self.definitions,
                )
            })?;
        let Some(paths) = self.declaring_files.get(key.queue_name.as_str()) else {
            return Some(ResolvedSite {
                queue_name: key.queue_name,
                queue_file: Some(key.queue_file),
                declaration: None,
                ambiguous_defaults: false,
            });
        };
        if paths.len() == 1 {
            return Some(ResolvedSite {
                queue_name: key.queue_name,
                queue_file: Some((*paths.first().expect("single declaring file")).clone()),
                declaration: None,
                ambiguous_defaults: false,
            });
        }
        let origin = local_named_origin(path, file, site.binding.as_ref(), &key.queue_name)
            .or_else(|| {
                imported_binding(file, site.binding.as_ref()?)
                    .and_then(|binding| {
                        crate::queue::factory_resolve::lookup_queue_origin(
                            self.facts,
                            self.resolver,
                            self.remapper,
                            path,
                            binding,
                        )
                    })
                    .filter(|(_, name)| name == &key.queue_name)
                    .map(|(origin_path, _)| origin_path)
            });
        if let Some(definition) = origin.as_ref().and_then(|origin| paths.get(origin)) {
            Some(ResolvedSite {
                queue_name: key.queue_name,
                queue_file: Some((*definition).clone()),
                declaration: None,
                ambiguous_defaults: false,
            })
        } else {
            Some(ResolvedSite {
                queue_name: key.queue_name,
                queue_file: None,
                declaration: None,
                ambiguous_defaults: true,
            })
        }
    }
}

fn local_named_origin(
    path: &Path,
    file: &FileFacts,
    binding: Option<&ProducerBinding>,
    queue_name: &str,
) -> Option<PathBuf> {
    let ProducerBinding::Named(name) = binding? else {
        return None;
    };
    (file.queue_bindings.get(name).map(String::as_str) == Some(queue_name))
        .then(|| path.to_path_buf())
}

fn imported_binding<'a>(file: &'a FileFacts, binding: &'a ProducerBinding) -> Option<&'a str> {
    match binding {
        ProducerBinding::Named(name) => Some(name),
        ProducerBinding::Local(declaration) => match file.queue_declarations.get(declaration)? {
            LocalQueue::ImportedQueue(name) => Some(name),
            _ => None,
        },
        ProducerBinding::Flow(_) | ProducerBinding::QueueName(_) | ProducerBinding::Unknown => None,
    }
}
