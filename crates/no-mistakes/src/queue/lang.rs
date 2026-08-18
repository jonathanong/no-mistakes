use crate::codebase::lang_frontends::{
    collect_all_lang_facts, lang_config_from_v2, lang_config_is_empty, matching_cluster,
    queue_globs_from_v2, topic_identity, LangFileFacts, QueueGlobMatchers,
};
use crate::config::v2::NoMistakesConfig;
use crate::queue::extract_model::{ProducerSite, WorkerSite};
use crate::queue::graph_model::{InternalProducer, InternalWorker};
use crate::queue::types::QueueKey;
use globset::GlobSet;
use std::path::{Path, PathBuf};

pub(super) fn language_queue_sites(
    root: &Path,
    session: &crate::codebase::analysis_session::AnalysisSession,
    config: &NoMistakesConfig,
    cli_filter: Option<&GlobSet>,
) -> (Vec<InternalProducer>, Vec<InternalWorker>) {
    let lang = lang_config_from_v2(config);
    let globs = queue_globs_from_v2(config);
    if lang_config_is_empty(&lang) && globs.enqueues.is_empty() && globs.workers.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let dataset = session.dataset(root);
    let all_files = dataset.paths_for(root);
    let sources = dataset.sources_for(root);
    let mut producers = Vec::new();
    let mut workers = Vec::new();
    if !lang_config_is_empty(&lang) {
        let collected = collect_all_lang_facts(root, &all_files, &lang, &sources);
        for map in crate::codebase::lang_frontends::each_lang_map(&collected) {
            for file in map.files.values() {
                extend_file(root, file, &globs, cli_filter, &mut producers, &mut workers);
            }
        }
    }
    if !globs.enqueues.is_empty() || !globs.workers.is_empty() {
        extend_kafka(
            root,
            &all_files,
            &sources,
            &globs,
            cli_filter,
            &mut producers,
            &mut workers,
        );
    }
    (producers, workers)
}

fn extend_file(
    root: &Path,
    file: &LangFileFacts,
    globs: &QueueGlobMatchers,
    cli_filter: Option<&GlobSet>,
    producers: &mut Vec<InternalProducer>,
    workers: &mut Vec<InternalWorker>,
) {
    if !cli_allows(root, &file.path, cli_filter) {
        return;
    }
    if let Some(cluster) = matching_cluster(root, &file.path, &globs.workers, globs) {
        for job in &file.queue_workers {
            workers.push(language_worker(root, &file.path, job, cluster.as_deref()));
        }
    }
    if let Some(cluster) = matching_cluster(root, &file.path, &globs.enqueues, globs) {
        for job in &file.queue_enqueues {
            producers.push(language_producer(root, &file.path, job, cluster.as_deref()));
        }
    }
}

fn extend_kafka(
    root: &Path,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    globs: &QueueGlobMatchers,
    cli_filter: Option<&GlobSet>,
    producers: &mut Vec<InternalProducer>,
    workers: &mut Vec<InternalWorker>,
) {
    for path in all_files {
        if !cli_allows(root, path, cli_filter) {
            continue;
        }
        let enqueue = matching_cluster(root, path, &globs.enqueues, globs);
        let worker = matching_cluster(root, path, &globs.workers, globs);
        if enqueue.is_none() && worker.is_none() {
            continue;
        }
        let Some((prod, cons)) = crate::codebase::lang_frontends::scan_kafka_file(path, sources)
        else {
            continue;
        };
        if let Some(cluster) = enqueue {
            for topic in prod {
                producers.push(language_producer(root, path, &topic, cluster.as_deref()));
            }
        }
        if let Some(cluster) = worker {
            for topic in cons {
                workers.push(language_worker(root, path, &topic, cluster.as_deref()));
            }
        }
    }
}

fn language_producer(
    root: &Path,
    path: &Path,
    job: &str,
    cluster: Option<&str>,
) -> InternalProducer {
    let identity = topic_identity(cluster, job);
    InternalProducer {
        site: ProducerSite {
            file: path.to_path_buf(),
            line: 0,
            binding: String::new(),
            job: Some(identity),
            raw_job: Some(job.to_string()),
        },
        queue: Some(queue_key(root, cluster)),
    }
}

fn language_worker(root: &Path, path: &Path, job: &str, cluster: Option<&str>) -> InternalWorker {
    InternalWorker {
        site: WorkerSite {
            file: path.to_path_buf(),
            line: 0,
            queue_name: Some(cluster.unwrap_or("default").to_string()),
            jobs: vec![topic_identity(cluster, job)],
            processor_specifier: None,
            processor_file: None,
            wildcard: false,
        },
        queue: Some(queue_key(root, cluster)),
    }
}

fn queue_key(root: &Path, cluster: Option<&str>) -> QueueKey {
    let name = cluster.unwrap_or("default");
    QueueKey {
        queue_file: root.join(name),
        queue_name: name.to_string(),
    }
}

fn cli_allows(root: &Path, path: &Path, filter: Option<&GlobSet>) -> bool {
    filter.is_none_or(|filter| filter.is_match(path.strip_prefix(root).unwrap_or(path)))
}

#[cfg(test)]
#[path = "lang_unit_tests.rs"]
mod unit_tests;
