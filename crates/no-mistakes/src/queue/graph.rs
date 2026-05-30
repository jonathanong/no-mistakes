use crate::queue::extract::FileFacts;
use crate::queue::graph_build::build_report;
use crate::queue::graph_model::{build_filter, InternalProducer, InternalWorker, ProjectReport};
use crate::queue::resolver::{load_tsconfig, resolve_import};
use crate::queue::source::discover_source_files;
use crate::queue::types::QueueKey;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RelatedDirection {
    Deps,
    Dependents,
    Both,
}

pub fn analyze_project(
    root: &Path,
    tsconfig_path: Option<&Path>,
    filters: &[String],
) -> anyhow::Result<ProjectReport> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let tsconfig = load_tsconfig(&root, tsconfig_path)?;
    let filter = build_filter(filters)?;
    let factory_names = load_factory_names(&root);
    let files = discover_source_files(&root)
        .into_iter()
        .filter(|path| {
            filter
                .as_ref()
                .is_none_or(|f| f.is_match(path.strip_prefix(&root).unwrap_or(path)))
        })
        .collect::<Vec<_>>();
    let mut context = crate::codebase::ts_source::facts::TsFactContext::new(&root);
    context.queue_project_factory_names = factory_names;
    let ts_facts = crate::codebase::ts_source::facts::collect_ts_facts_with_context(
        &files,
        crate::codebase::ts_source::facts::TsFactPlan {
            queue_project: true,
            ..Default::default()
        },
        &context,
    );
    let facts = queue_project_facts_from_ts(&ts_facts, filter.as_ref(), &root);

    let queue_defs = queue_definitions(&facts);
    let producers = resolve_producers(&root, &facts, &queue_defs, &tsconfig);
    let workers = resolve_workers(&root, &facts, &queue_defs, &tsconfig);
    Ok(build_report(&root, producers, workers, &facts))
}

fn queue_project_facts_from_ts(
    ts_facts: &crate::codebase::ts_source::facts::TsFactMap,
    filter: Option<&globset::GlobSet>,
    root: &Path,
) -> HashMap<PathBuf, FileFacts> {
    ts_facts
        .iter()
        .filter_map(|(path, facts)| {
            if let Some(filter) = filter {
                let rel = path.strip_prefix(root).unwrap_or(path);
                if !filter.is_match(rel) {
                    return None;
                }
            }
            facts
                .queue_project
                .as_ref()
                .map(|queue| (path.clone(), queue.clone()))
        })
        .collect()
}

fn load_factory_names(root: &Path) -> Vec<String> {
    crate::config::v2::load_v2_config(root, None)
        .map(|config| config.queues.factories)
        .unwrap_or_default()
}

pub fn analyze_project_with_facts(
    root: &Path,
    tsconfig_path: Option<&Path>,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> anyhow::Result<ProjectReport> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let root = root.as_path();
    let tsconfig = load_tsconfig(root, tsconfig_path)?;
    let filter = build_filter(filters)?;
    let ts_facts = shared.ts_facts();
    let facts = queue_project_facts_from_ts(&ts_facts, filter.as_ref(), root);
    let queue_defs = queue_definitions(&facts);
    let producers = resolve_producers(root, &facts, &queue_defs, &tsconfig);
    let workers = resolve_workers(root, &facts, &queue_defs, &tsconfig);
    Ok(build_report(root, producers, workers, &facts))
}

fn queue_definitions(
    facts: &HashMap<PathBuf, FileFacts>,
) -> HashMap<PathBuf, HashMap<String, String>> {
    facts
        .iter()
        .map(|(path, facts)| (path.clone(), facts.queue_exports.clone()))
        .collect()
}

fn resolve_producers(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    queue_defs: &HashMap<PathBuf, HashMap<String, String>>,
    tsconfig: &crate::queue::resolver::TsConfig,
) -> Vec<InternalProducer> {
    let mut out = Vec::new();
    for (path, facts) in facts {
        let local = local_queues(path, root, facts, queue_defs, tsconfig);
        for site in &facts.producers {
            let queue = local.get(&site.binding).cloned();
            out.push(InternalProducer {
                site: site.clone(),
                queue,
            });
        }
    }
    out
}

fn resolve_workers(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    queue_defs: &HashMap<PathBuf, HashMap<String, String>>,
    tsconfig: &crate::queue::resolver::TsConfig,
) -> Vec<InternalWorker> {
    let by_name = queue_defs
        .iter()
        .flat_map(|(file, exports)| exports.values().map(|name| (name.clone(), file.clone())))
        .collect::<HashMap<_, _>>();
    let mut out = Vec::new();
    for (path, facts) in facts {
        for site in &facts.workers {
            let queue = site.queue_name.as_ref().and_then(|name| {
                by_name.get(name).map(|file| QueueKey {
                    queue_file: file.clone(),
                    queue_name: name.clone(),
                })
            });
            let mut site = site.clone();
            site.processor_file = site
                .processor_specifier
                .as_ref()
                .and_then(|spec| resolve_import(spec, path, root, tsconfig));
            out.push(InternalWorker { site, queue });
        }
    }
    out
}

fn local_queues(
    path: &Path,
    root: &Path,
    facts: &FileFacts,
    queue_defs: &HashMap<PathBuf, HashMap<String, String>>,
    tsconfig: &crate::queue::resolver::TsConfig,
) -> HashMap<String, QueueKey> {
    let mut map = facts
        .queue_bindings
        .iter()
        .map(|(binding, queue_name)| {
            (
                binding.clone(),
                QueueKey {
                    queue_file: path.to_path_buf(),
                    queue_name: queue_name.clone(),
                },
            )
        })
        .collect::<HashMap<_, _>>();
    for import in &facts.imports {
        let Some(resolved) = resolve_import(&import.source, path, root, tsconfig) else {
            continue;
        };
        if let Some(exports) = queue_defs.get(&resolved) {
            if let Some(queue_name) = exports.get(&import.imported) {
                map.insert(
                    import.local.clone(),
                    QueueKey {
                        queue_file: resolved,
                        queue_name: queue_name.clone(),
                    },
                );
            }
        }
    }
    map
}
