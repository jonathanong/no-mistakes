use crate::queue::extract::FileFacts;
use crate::queue::graph_build::build_report;
use crate::queue::graph_model::{build_filter, InternalProducer, InternalWorker, ProjectReport};
use crate::queue::resolver::{load_tsconfig_from_visible, resolve_import_from_visible};
use crate::queue::types::QueueKey;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod fact_sources;
use fact_sources::{queue_project_facts_from_shared, queue_project_facts_from_ts};

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
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible_paths = snapshot.paths_for(&root);
    let visible_files =
        crate::codebase::ts_source::discover_files_from_visible(&root, &[], &visible_paths);
    let visible_set = visible_files.iter().cloned().collect();
    let tsconfig = load_tsconfig_from_visible(&root, tsconfig_path, &visible_files)?;
    let filter = build_filter(filters)?;
    let factory_names = crate::config::v2::load_v2_config_from_visible(&root, None, &visible_paths)
        .map(|config| config.queues.factories)
        .unwrap_or_default();
    let files = visible_files
        .into_iter()
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    crate::codebase::ts_source::TS_JS_EXTENSIONS.contains(&extension)
                })
        })
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
    let facts = queue_project_facts_from_ts(ts_facts, filter.as_ref(), &root);

    let queue_defs = queue_definitions(&facts);
    let producers = resolve_producers(&root, &facts, &queue_defs, &tsconfig, &visible_set);
    let workers = resolve_workers(&root, &facts, &queue_defs, &tsconfig, &visible_set);
    Ok(build_report(&root, producers, workers, &facts))
}

pub fn analyze_project_with_facts(
    root: &Path,
    tsconfig_path: Option<&Path>,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> anyhow::Result<ProjectReport> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let root = root.as_path();
    let tsconfig = load_tsconfig_from_visible(root, tsconfig_path, shared.files())?;
    let filter = build_filter(filters)?;
    let facts = queue_project_facts_from_shared(shared, filter.as_ref(), root);
    let visible_files = shared.files().iter().cloned().collect();
    let queue_defs = queue_definitions(&facts);
    let producers = resolve_producers(root, &facts, &queue_defs, &tsconfig, &visible_files);
    let workers = resolve_workers(root, &facts, &queue_defs, &tsconfig, &visible_files);
    Ok(build_report(root, producers, workers, &facts))
}

/// Analyze shared queue facts with a TypeScript config already prepared by the caller.
///
/// The aggregate check uses this entrypoint so every domain shares the request's
/// gitignore-aware config selection. Standalone queue commands continue to use
/// [`analyze_project_with_facts`] and retain their existing config discovery behavior.
#[doc(hidden)]
pub fn analyze_project_with_prepared_facts(
    root: &Path,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    filters: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> anyhow::Result<ProjectReport> {
    let root = root.canonicalize().unwrap_or(root.to_path_buf());
    let root = root.as_path();
    let tsconfig = crate::queue::resolver::TsConfig::from(tsconfig);
    let filter = build_filter(filters)?;
    let facts = queue_project_facts_from_shared(shared, filter.as_ref(), root);
    let visible_files = shared.files().iter().cloned().collect();
    let queue_defs = queue_definitions(&facts);
    let producers = resolve_producers(root, &facts, &queue_defs, &tsconfig, &visible_files);
    let workers = resolve_workers(root, &facts, &queue_defs, &tsconfig, &visible_files);
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
    visible_files: &std::collections::HashSet<PathBuf>,
) -> Vec<InternalProducer> {
    let mut out = Vec::new();
    for (path, facts) in facts {
        let local = local_queues(path, root, facts, queue_defs, tsconfig, visible_files);
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
    visible_files: &std::collections::HashSet<PathBuf>,
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
            site.processor_file = site.processor_specifier.as_ref().and_then(|spec| {
                resolve_import_from_visible(spec, path, root, tsconfig, visible_files)
            });
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
    visible_files: &std::collections::HashSet<PathBuf>,
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
        let Some(resolved) =
            resolve_import_from_visible(&import.source, path, root, tsconfig, visible_files)
        else {
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
