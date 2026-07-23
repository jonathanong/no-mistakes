fn parsed_workflows_for_graph(
    root: &Path,
    all_files: &[PathBuf],
    ci: &crate::config::v2::schema::CiConfig,
) -> crate::codebase::ci_workflows::ParsedWorkflowSet {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let workflow_dirs: HashSet<PathBuf> = ci
        .workflow_dirs
        .iter()
        .map(|directory| crate::codebase::ts_resolver::normalize_path(&root.join(directory)))
        .collect();
    let paths = all_files.iter().filter(|path| {
        path.parent()
            .is_some_and(|parent| workflow_dirs.contains(parent))
            && path
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .is_some_and(|extension| {
                    extension.eq_ignore_ascii_case("yml")
                        || extension.eq_ignore_ascii_case("yaml")
                })
    });
    crate::codebase::ci_workflows::ParsedWorkflowSet::from_paths(&root, paths.cloned())
}

fn collect_workflow_topology_edges(
    root: &Path,
    all_files: &[PathBuf],
    ci: &crate::config::v2::schema::CiConfig,
    parsed: &crate::codebase::ci_workflows::ParsedWorkflowSet,
    topology: &crate::codebase::workflow_topology::model::WorkflowTopology,
) -> Vec<Edge> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let universe: HashSet<PathBuf> = all_files.iter().cloned().collect();
    let workflow_files: HashSet<PathBuf> = topology
        .workflows
        .iter()
        .map(|workflow| {
            crate::codebase::ts_resolver::normalize_path(&root.join(&workflow.path))
        })
        .collect();
    let action_dirs: Vec<PathBuf> = ci
        .action_dirs
        .iter()
        .map(|directory| crate::codebase::ts_resolver::normalize_path(&root.join(directory)))
        .collect();
    let mut edges = Vec::new();
    let mut jobs = HashMap::new();
    let mut steps = HashMap::new();

    for job in &topology.jobs {
        let workflow_file =
            crate::codebase::ts_resolver::normalize_path(&root.join(&job.workflow_id));
        if !universe.contains(&workflow_file) {
            continue;
        }
        let job_node = NodeId::WorkflowJob {
            workflow_file: workflow_file.clone(),
            job: job.key.clone(),
        };
        jobs.insert(job.id.clone(), job_node.clone());
        edges.push((
            NodeId::File(workflow_file.clone()),
            job_node.clone(),
            EdgeKind::WorkflowJob,
        ));
        for step in &job.steps {
            let step_node = NodeId::WorkflowStep {
                workflow_file: workflow_file.clone(),
                job: job.key.clone(),
                step: step.index as usize,
            };
            steps.insert((job.id.clone(), step.index as usize), step_node.clone());
            edges.push((
                job_node.clone(),
                step_node.clone(),
                EdgeKind::WorkflowStep,
            ));
            if let Some(target) = step.uses.as_deref().and_then(|target| {
                resolve_local_action_descriptor(&root, target, &universe, &action_dirs)
            }) {
                edges.push((
                    step_node,
                    NodeId::File(target),
                    EdgeKind::WorkflowUses,
                ));
            }
        }
    }

    for edge in &topology.edges {
        use crate::codebase::workflow_topology::model::WorkflowTopologyEdge;
        match edge {
            WorkflowTopologyEdge::Needs(edge) => {
                if let (Some(from), Some(to)) = (jobs.get(&edge.from), jobs.get(&edge.to)) {
                    edges.push((from.clone(), to.clone(), EdgeKind::WorkflowNeeds));
                }
            }
            WorkflowTopologyEdge::Calls(edge) => {
                let Some(target) = edge.to.as_deref() else {
                    continue;
                };
                let target = crate::codebase::ts_resolver::normalize_path(&root.join(target));
                if let Some(from) = jobs
                    .get(&edge.from)
                    .filter(|_| workflow_files.contains(&target))
                {
                    edges.push((
                        from.clone(),
                        NodeId::File(target),
                        EdgeKind::WorkflowUses,
                    ));
                }
            }
            WorkflowTopologyEdge::Artifact(edge) => {
                let from = steps.get(&(edge.from.clone(), edge.producer_step as usize));
                let to = steps.get(&(edge.to.clone(), edge.consumer_step as usize));
                if let (Some(from), Some(to)) = (from, to) {
                    edges.push((from.clone(), to.clone(), EdgeKind::WorkflowArtifact));
                }
            }
            WorkflowTopologyEdge::WorkflowRun(_) => {}
        }
    }

    add_workflow_run_edges(&root, &universe, parsed, &jobs, &steps, &mut edges);
    edges.sort();
    edges.dedup();
    edges
}

fn add_workflow_run_edges(
    root: &Path,
    universe: &HashSet<PathBuf>,
    parsed: &crate::codebase::ci_workflows::ParsedWorkflowSet,
    jobs: &HashMap<String, NodeId>,
    steps: &HashMap<(String, usize), NodeId>,
    edges: &mut Vec<Edge>,
) {
    let mut all_files: Vec<PathBuf> = universe.iter().cloned().collect();
    all_files.sort();
    let bins = collect_cargo_bins(root, &all_files);
    let mut resolver = WorkflowRunResolver::new(root, universe, &bins);
    for document in &parsed.documents {
        let Ok(workflow) = document.value.as_ref() else {
            continue;
        };
        let Some(raw_jobs) = workflow.get("jobs").and_then(serde_yaml::Value::as_mapping) else {
            continue;
        };
        for (job_key, raw_job) in raw_jobs {
            let Some(job_key) = job_key.as_str() else {
                continue;
            };
            let job_id = format!("{}#{job_key}", document.path);
            if !jobs.contains_key(&job_id) {
                continue;
            }
            let Some(raw_steps) = raw_job.get("steps").and_then(serde_yaml::Value::as_sequence)
            else {
                continue;
            };
            for (position, raw_step) in raw_steps.iter().enumerate() {
                let Some(run) = raw_step.get("run").and_then(serde_yaml::Value::as_str) else {
                    continue;
                };
                let Some(step_node) = steps.get(&(job_id.clone(), position)) else {
                    continue;
                };
                let Some(working_directory) =
                    workflow_run_working_directory(root, workflow, raw_job, raw_step)
                else {
                    continue;
                };
                for target in resolver.resolve(run, &working_directory) {
                    edges.push((
                        step_node.clone(),
                        NodeId::File(target),
                        EdgeKind::WorkflowRun,
                    ));
                }
            }
        }
    }
}
