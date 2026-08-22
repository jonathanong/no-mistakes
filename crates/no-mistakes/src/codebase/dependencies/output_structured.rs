use serde::Serialize;

#[derive(Serialize)]
#[serde(untagged)]
enum OutputNode {
    File(OutputFile),
    Symbol(OutputSymbol),
    QueueJob(OutputQueueJob),
    WorkflowJob(OutputWorkflowJob),
    WorkflowStep(OutputWorkflowStep),
    TrpcProcedure(OutputTrpcProcedure),
    Module(OutputModule),
}

#[derive(Serialize)]
struct OutputFile {
    path: String,
    depth: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    via: Vec<&'static str>,
}

#[derive(Serialize)]
struct OutputSymbol {
    file: String,
    symbol: String,
    depth: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    via: Vec<&'static str>,
}

#[derive(Serialize)]
struct OutputQueueJob {
    #[serde(rename = "queueFile")]
    queue_file: String,
    job: String,
    depth: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    via: Vec<&'static str>,
}

#[derive(Serialize)]
struct OutputModule {
    module: String,
    depth: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    via: Vec<&'static str>,
}

#[derive(Serialize)]
struct OutputWorkflowJob {
    #[serde(rename = "workflowFile")]
    workflow_file: String,
    job: String,
    depth: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    via: Vec<&'static str>,
}

#[derive(Serialize)]
struct OutputWorkflowStep {
    #[serde(rename = "workflowFile")]
    workflow_file: String,
    job: String,
    step: usize,
    depth: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    via: Vec<&'static str>,
}

#[derive(Serialize)]
struct OutputTrpcProcedure {
    #[serde(rename = "routerFile")]
    router_file: String,
    procedure: String,
    depth: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    via: Vec<&'static str>,
}

#[derive(Serialize)]
struct Output {
    roots: Vec<String>,
    files: Vec<OutputNode>,
}

#[derive(Serialize)]
struct OutputWithDiagnostics<'a> {
    roots: Vec<String>,
    files: Vec<OutputNode>,
    diagnostics: &'a [crate::codebase::ts_resolver::TsConfigDiagnostic],
    tsconfig_provenance: &'a [crate::codebase::ts_resolver::TsConfigProvenance],
}

fn build_output_with_diagnostics<'a>(
    roots: &[String],
    entries: &[NodeEntry],
    root_dir: &Path,
    diagnostics: &'a [crate::codebase::ts_resolver::TsConfigDiagnostic],
    provenance: &'a [crate::codebase::ts_resolver::TsConfigProvenance],
) -> OutputWithDiagnostics<'a> {
    let Output { roots, files } = build_output(roots, entries, root_dir);
    OutputWithDiagnostics {
        roots,
        files,
        diagnostics,
        tsconfig_provenance: provenance,
    }
}

fn build_output(roots: &[String], entries: &[NodeEntry], root_dir: &Path) -> Output {
    Output {
        roots: roots.to_vec(),
        files: entries
            .iter()
            .map(|entry| {
                let via: Vec<&'static str> = entry.via.iter().map(|k| k.as_str()).collect();
                match &entry.node {
                    NodeId::File(path) => {
                        let rel = path.strip_prefix(root_dir).unwrap_or(path);
                        OutputNode::File(OutputFile {
                            path: rel.to_string_lossy().into_owned(),
                            depth: entry.depth,
                            via,
                        })
                    }
                    NodeId::Symbol { file, symbol } => {
                        let rel = file.strip_prefix(root_dir).unwrap_or(file);
                        OutputNode::Symbol(OutputSymbol {
                            file: rel.to_string_lossy().into_owned(),
                            symbol: symbol.to_string(),
                            depth: entry.depth,
                            via,
                        })
                    }
                    NodeId::QueueJob { queue_file, job } => {
                        let rel = queue_file
                            .strip_prefix(root_dir)
                            .unwrap_or(queue_file.as_ref());
                        OutputNode::QueueJob(OutputQueueJob {
                            queue_file: rel.to_string_lossy().into_owned(),
                            job: job.to_string(),
                            depth: entry.depth,
                            via,
                        })
                    }
                    NodeId::WorkflowJob { workflow_file, job } => {
                        let rel = workflow_file
                            .strip_prefix(root_dir)
                            .unwrap_or(workflow_file.as_ref());
                        OutputNode::WorkflowJob(OutputWorkflowJob {
                            workflow_file: rel.to_string_lossy().into_owned(),
                            job: job.to_string(),
                            depth: entry.depth,
                            via,
                        })
                    }
                    NodeId::WorkflowStep {
                        workflow_file,
                        job,
                        step,
                    } => {
                        let rel = workflow_file
                            .strip_prefix(root_dir)
                            .unwrap_or(workflow_file.as_ref());
                        OutputNode::WorkflowStep(OutputWorkflowStep {
                            workflow_file: rel.to_string_lossy().into_owned(),
                            job: job.to_string(),
                            step: *step,
                            depth: entry.depth,
                            via,
                        })
                    }
                    NodeId::Module(specifier) => OutputNode::Module(OutputModule {
                        module: specifier.to_string(),
                        depth: entry.depth,
                        via,
                    }),
                    NodeId::TrpcProcedure {
                        router_file,
                        procedure,
                    } => {
                        let rel = router_file
                            .strip_prefix(root_dir)
                            .unwrap_or(router_file.as_ref());
                        OutputNode::TrpcProcedure(OutputTrpcProcedure {
                            router_file: rel.to_string_lossy().into_owned(),
                            procedure: procedure.to_string(),
                            depth: entry.depth,
                            via,
                        })
                    }
                }
            })
            .collect(),
    }
}
