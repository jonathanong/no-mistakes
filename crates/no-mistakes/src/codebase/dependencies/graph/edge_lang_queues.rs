struct CompiledQueueGlobs {
    matchers: Vec<(globset::GlobMatcher, String)>,
}

fn compile_queue_globs(globs: &[String]) -> CompiledQueueGlobs {
    CompiledQueueGlobs {
        matchers: globs
            .iter()
            .filter_map(|glob| {
                globset::Glob::new(glob)
                    .ok()
                    .map(|compiled| (compiled.compile_matcher(), glob.clone()))
            })
            .collect(),
    }
}

fn emit_queue_edges(
    root: &Path,
    facts: &LangFactMap,
    options: &GraphConfigOptions,
    edges: &mut Vec<Edge>,
) {
    let worker_globs = compile_queue_globs(&options.queue_workers);
    let enqueue_globs = compile_queue_globs(&options.queue_enqueues);
    let mut workers: std::collections::HashMap<String, std::collections::BTreeSet<PathBuf>> =
        std::collections::HashMap::new();
    for file in facts.files.values() {
        let Some(cluster) = matching_queue_cluster(root, &file.path, &worker_globs, options)
        else {
            continue;
        };
        for job in &file.queue_workers {
            workers
                .entry(topic_identity(cluster.as_deref(), job))
                .or_default()
                .insert(file.path.clone());
        }
    }
    for file in facts.files.values() {
        let Some(cluster) = matching_queue_cluster(root, &file.path, &enqueue_globs, options)
        else {
            continue;
        };
        for job in &file.queue_enqueues {
            let identity = topic_identity(cluster.as_deref(), job);
            let node = NodeId::queue_job(&file.path, identity.clone());
            edges.push((
                NodeId::file(&file.path),
                node.clone(),
                EdgeKind::QueueEnqueue,
            ));
            if let Some(targets) = workers.get(&identity) {
                for worker in targets {
                    edges.push((node.clone(), NodeId::file(worker), EdgeKind::QueueWorker));
                }
            }
        }
    }
}

fn matching_queue_cluster(
    root: &Path,
    path: &Path,
    compiled: &CompiledQueueGlobs,
    options: &GraphConfigOptions,
) -> Option<Option<String>> {
    if compiled.matchers.is_empty() {
        return None;
    }
    let rel = path.strip_prefix(root).unwrap_or(path);
    compiled.matchers.iter().find_map(|(matcher, glob)| {
        matcher.is_match(rel).then(|| match options.queue_glob_clusters.get(glob) {
            Some(cluster) => cluster.clone(),
            None => options.queue_cluster.clone(),
        })
    })
}

fn emit_kafka_edges(
    root: &Path,
    all_files: &[PathBuf],
    options: &GraphConfigOptions,
    edges: &mut Vec<Edge>,
) {
    let enqueue_globs = compile_queue_globs(&options.queue_enqueues);
    let worker_globs = compile_queue_globs(&options.queue_workers);
    let mut produces = Vec::new();
    let mut consumes = Vec::new();
    for path in all_files {
        let enqueue = matching_queue_cluster(root, path, &enqueue_globs, options);
        let worker = matching_queue_cluster(root, path, &worker_globs, options);
        if enqueue.is_none() && worker.is_none() {
            continue;
        }
        let Some((prod, cons)) = scan_kafka_file(path) else {
            continue;
        };
        if let Some(cluster) = enqueue {
            produces.push((path.clone(), prod, cluster));
        }
        if let Some(cluster) = worker {
            consumes.push((path.clone(), cons, cluster));
        }
    }
    let mut workers: std::collections::HashMap<String, std::collections::BTreeSet<PathBuf>> =
        std::collections::HashMap::new();
    for (path, topics, cluster) in &consumes {
        for topic in topics {
            workers
                .entry(topic_identity(cluster.as_deref(), topic))
                .or_default()
                .insert(path.clone());
        }
    }
    for (path, topics, cluster) in produces {
        for topic in topics {
            let identity = topic_identity(cluster.as_deref(), &topic);
            let node = NodeId::queue_job(&path, identity.clone());
            edges.push((
                NodeId::file(&path),
                node.clone(),
                EdgeKind::QueueEnqueue,
            ));
            if let Some(targets) = workers.get(&identity) {
                for worker in targets {
                    edges.push((node.clone(), NodeId::file(worker), EdgeKind::QueueWorker));
                }
            }
        }
    }
}

#[cfg(test)]
fn matches_any_naive(rel: &Path, globs: &[String]) -> bool {
    globs.iter().any(|glob| {
        globset::Glob::new(glob)
            .ok()
            .is_some_and(|compiled| compiled.compile_matcher().is_match(rel))
    })
}
