fn emit_route_edges(facts: &LangFactMap, edges: &mut Vec<Edge>) {
    for file in facts.files.values() {
        for (_, handler) in &file.route_handlers {
            for name in route_handler_names(handler) {
                if let Some(targets) = facts.declarations.get(&name) {
                    let scoped: std::collections::BTreeSet<_> = targets
                        .iter()
                        .filter(|target| {
                            file.package.is_none()
                                || facts
                                    .files
                                    .get(*target)
                                    .and_then(|other| other.package.as_ref())
                                    == file.package.as_ref()
                        })
                        .cloned()
                        .collect();
                    push_file_edges(edges, &file.path, &scoped, EdgeKind::RouteRef);
                }
            }
        }
    }
}

fn route_handler_names(handler: &str) -> Vec<String> {
    let trimmed = handler.replace(['\'', '"', ' '], "");
    if let Some((controller, _)) = trimmed.split_once('#') {
        return rails_controller_names(controller);
    }
    if let Some((class, _)) = trimmed.split_once("::") {
        return vec![class.rsplit('\\').next().unwrap_or(class).to_string()];
    }
    let view = trimmed
        .strip_suffix("()")
        .unwrap_or(trimmed.as_str())
        .strip_suffix(".as_view")
        .unwrap_or(trimmed.as_str());
    vec![view.rsplit('.').next().unwrap_or(view).to_string()]
}

fn rails_controller_names(controller: &str) -> Vec<String> {
    let parts: Vec<&str> = controller.split('/').filter(|part| !part.is_empty()).collect();
    let last = parts.last().copied().unwrap_or(controller);
    let mut class = last.to_string();
    if !class.ends_with("Controller") {
        class.push_str("Controller");
    }
    let class = snake_to_pascal(&class);
    if parts.len() <= 1 {
        return vec![class];
    }
    let namespace = parts[..parts.len() - 1]
        .iter()
        .map(|part| snake_to_pascal(part))
        .collect::<Vec<_>>()
        .join("::");
    vec![format!("{namespace}::{class}"), class]
}

fn snake_to_pascal(name: &str) -> String {
    name.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let first = chars.next().expect("non-empty");
            first.to_ascii_uppercase().to_string() + chars.as_str()
        })
        .collect()
}

fn emit_kafka_edges(
    root: &Path,
    all_files: &[PathBuf],
    options: &GraphConfigOptions,
    edges: &mut Vec<Edge>,
) {
    let cluster = options.queue_cluster.as_deref();
    let mut produces = Vec::new();
    let mut consumes = Vec::new();
    for path in all_files {
        let rel = path.strip_prefix(root).unwrap_or(path);
        let rel_s = rel.to_string_lossy();
        let enqueue = matches_any(&rel_s, &options.queue_enqueues);
        let worker = matches_any(&rel_s, &options.queue_workers);
        if !enqueue && !worker {
            continue;
        }
        let Some((prod, cons)) = scan_kafka_file(path) else {
            continue;
        };
        if enqueue {
            produces.push((path.clone(), prod));
        }
        if worker {
            consumes.push((path.clone(), cons));
        }
    }
    let mut workers: std::collections::HashMap<String, std::collections::BTreeSet<PathBuf>> =
        std::collections::HashMap::new();
    for (path, topics) in &consumes {
        for topic in topics {
            workers
                .entry(topic_identity(cluster, topic))
                .or_default()
                .insert(path.clone());
        }
    }
    for (path, topics) in produces {
        for topic in topics {
            let identity = topic_identity(cluster, &topic);
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

fn matches_any(rel: &str, globs: &[String]) -> bool {
    globs.iter().any(|glob| {
        globset::Glob::new(glob)
            .ok()
            .is_some_and(|g| g.compile_matcher().is_match(rel))
    })
}
