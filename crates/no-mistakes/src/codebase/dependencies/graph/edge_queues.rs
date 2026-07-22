/// A typed relationship produced by the dashboard queue semantics.
///
/// This stays separate from `queue::RelationshipEdge`: dashboard analysis only
/// exposes jobs that have both an enqueue site and a matching worker, whereas
/// project queue analysis reports its own broader producer/worker model.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DashboardQueueRelationship {
    from: NodeId,
    to: NodeId,
    kind: EdgeKind,
}

impl DashboardQueueRelationship {
    fn new(from: NodeId, to: NodeId, kind: EdgeKind) -> Self {
        Self { from, to, kind }
    }

    fn into_edge(self) -> Edge {
        (self.from, self.to, self.kind)
    }
}

fn collect_dashboard_queue_relationships(
    root: &Path,
    resolver: &dyn ImportResolution,
    graph_files: &GraphFiles,
    facts: Option<&dyn TsFactLookup>,
    config_options: Option<&GraphConfigOptions>,
) -> Vec<DashboardQueueRelationship> {
    use globset::GlobBuilder;

    let Some(config_options) = config_options else {
        return Vec::new();
    };
    let opts = &config_options.queue;
    let files = graph_files.indexable();

    if opts.queue_pattern.is_empty() || opts.factory_specifier.is_empty() {
        return Vec::new();
    }

    let glob = match GlobBuilder::new(&opts.queue_pattern)
        .literal_separator(false)
        .build()
    {
        Ok(g) => g,
        Err(_) => return Vec::new(),
    };
    let mut gb = globset::GlobSetBuilder::new();
    gb.add(glob);
    let gs = gb
        .build()
        .expect("globset with one validated queue pattern should build");

    // Phase 1: Find queue-def files and their queue names.
    // queue_name → def_file  (only queues with string-literal names)
    let mut queue_name_to_def: HashMap<String, PathBuf> = HashMap::new();
    // def_file → queue_name (for reverse lookup)
    let mut def_to_queue_name: HashMap<PathBuf, String> = HashMap::new();

    for path in files {
        let rel = path
            .strip_prefix(root)
            .expect("queue files are rooted under the graph root");
        if !gs.is_match(rel) {
            continue;
        }
        let Some((create_line, queue_name)) = facts
            .and_then(|facts| facts.get_ts_facts(path))
            .map(|file_facts| (file_facts.queue_create_line, file_facts.queue_name.clone()))
        else {
            continue;
        };
        if create_line.is_none() {
            continue;
        }
        if let Some(queue_name) = queue_name {
            queue_name_to_def.insert(queue_name.clone(), path.clone());
            def_to_queue_name.insert(path.clone(), queue_name);
        }
    }

    if queue_name_to_def.is_empty() {
        return Vec::new();
    }

    // Phase 2: For each file, extract queue usage. Collect:
    //   - EnqueueSites: (queue_def_file, job_name) per source file
    //   - WorkerSites: (queue_def_file, processor_file, job_names) per source file

    // enqueue_sites: (source_file, queue_def_file, job_name)
    let mut enqueue_sites: Vec<(PathBuf, PathBuf, String)> = Vec::new();
    // worker_sites: (worker_file, queue_def_file, processor_file, job_names)
    let mut worker_sites: Vec<(PathBuf, PathBuf, PathBuf, Vec<String>)> = Vec::new();
    let mut processor_job_names: HashMap<PathBuf, Vec<String>> = HashMap::new();

    let queue_def_paths: HashSet<PathBuf> = def_to_queue_name.keys().cloned().collect();

    for path in files {
        let Some(usage) = facts
            .and_then(|facts| facts.get_ts_facts(path))
            .and_then(|file_facts| file_facts.queue_usage.as_ref())
        else {
            continue;
        };

        // Resolve which imports come from queue-def files.
        // Build: local_binding → queue_def_file
        let mut binding_to_queue_def: HashMap<String, PathBuf> = HashMap::new();
        for (local_binding, import_spec) in &usage.imports {
            if let Some(resolved) = resolver
                .resolve(import_spec, path)
                .and_then(|resolved| graph_files.visible_path(&resolved))
            {
                if queue_def_paths.contains(resolved) {
                    binding_to_queue_def.insert(local_binding.clone(), resolved.to_path_buf());
                }
            }
        }

        // Enqueue sites.
        for call in &usage.enqueue_calls {
            if let (Some(queue_def), Some(job)) =
                (binding_to_queue_def.get(&call.binding), &call.job)
            {
                enqueue_sites.push((path.clone(), queue_def.clone(), job.clone()));
            }
        }

        // Worker registrations.
        for worker in &usage.worker_declarations {
            let queue_def = worker
                .queue_name
                .as_ref()
                .and_then(|name| queue_name_to_def.get(name))
                .cloned();
            let processors_file = worker
                .processors_specifier
                .as_ref()
                .and_then(|spec| resolver.resolve(spec, path))
                .and_then(|resolved| graph_files.visible_path(&resolved))
                .map(Path::to_path_buf);
            let (Some(queue_def), Some(processors_file)) = (queue_def, processors_file) else {
                continue;
            };

            let job_names = if let Some(job_names) = processor_job_names.get(&processors_file) {
                job_names.clone()
            } else {
                let job_names =
                    extract_processor_job_names(&processors_file, facts).unwrap_or_default();
                processor_job_names.insert(processors_file.clone(), job_names.clone());
                job_names
            };

            if !job_names.is_empty() {
                worker_sites.push((path.clone(), queue_def, processors_file, job_names));
            }
        }
    }

    // Phase 3: Build QueueJob nodes for matched (queue, job) pairs.
    // A job is "matched" if it appears in BOTH an enqueue site AND a worker.
    // Build index: (queue_def, job) → [enqueue_files]
    let mut enqueue_index: HashMap<(PathBuf, String), Vec<PathBuf>> = HashMap::new();
    for (src, queue_def, job) in &enqueue_sites {
        enqueue_index
            .entry((queue_def.clone(), job.clone()))
            .or_default()
            .push(src.clone());
    }

    let mut relationships = Vec::new();
    for (worker_file, queue_def, processor_file, job_names) in &worker_sites {
        for job in job_names {
            let key = (queue_def.clone(), job.clone());
            let Some(enqueue_files) = enqueue_index.get(&key) else {
                continue;
            };

            let queue_job = NodeId::QueueJob {
                queue_file: queue_def.clone(),
                job: job.clone(),
            };

            // Enqueue site → QueueJob.
            for enqueue_file in enqueue_files {
                relationships.push(DashboardQueueRelationship::new(
                    NodeId::File(enqueue_file.clone()),
                    queue_job.clone(),
                    EdgeKind::QueueEnqueue,
                ));
            }

            // QueueJob → processor file.
            relationships.push(DashboardQueueRelationship::new(
                queue_job.clone(),
                NodeId::File(processor_file.clone()),
                EdgeKind::QueueWorker,
            ));
            if worker_file != processor_file {
                relationships.push(DashboardQueueRelationship::new(
                    queue_job.clone(),
                    NodeId::File(worker_file.clone()),
                    EdgeKind::QueueWorker,
                ));
            }
        }
    }
    relationships
}

fn collect_queue_edges(
    root: &Path,
    resolver: &dyn ImportResolution,
    graph_files: &GraphFiles,
    facts: Option<&dyn TsFactLookup>,
    config_options: Option<&GraphConfigOptions>,
) -> Vec<Edge> {
    collect_dashboard_queue_relationships(root, resolver, graph_files, facts, config_options)
        .into_iter()
        .map(DashboardQueueRelationship::into_edge)
        .collect()
}
