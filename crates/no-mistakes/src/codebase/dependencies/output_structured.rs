use serde::Serialize;

fn edge_kind_str(k: EdgeKind) -> &'static str {
    match k {
        EdgeKind::Import => "import",
        EdgeKind::TypeImport => "type-import",
        EdgeKind::DynamicImport => "dynamic-import",
        EdgeKind::Require => "require",
        EdgeKind::TestOf => "test",
        EdgeKind::RouteRef => "route",
        EdgeKind::QueueEnqueue => "queue-enqueue",
        EdgeKind::QueueWorker => "queue-worker",
        EdgeKind::RouteTest => "route-test",
        EdgeKind::Layout => "layout",
        EdgeKind::MarkdownLink => "md",
        EdgeKind::WorkspaceImport => "workspace",
        EdgeKind::PackageDependency => "package",
        EdgeKind::CiInvocation => "ci",
        EdgeKind::HttpCall => "http",
        EdgeKind::ProcessSpawn => "process",
        EdgeKind::AssetImport => "asset",
        EdgeKind::ReactRender => "react-render",
        EdgeKind::Selector => "selector",
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum OutputNode {
    File(OutputFile),
    Symbol(OutputSymbol),
    QueueJob(OutputQueueJob),
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
struct Output {
    roots: Vec<String>,
    files: Vec<OutputNode>,
}

fn build_output(roots: &[String], entries: &[NodeEntry], root_dir: &Path) -> Output {
    Output {
        roots: roots.to_vec(),
        files: entries
            .iter()
            .map(|entry| {
                let via: Vec<&'static str> = entry.via.iter().map(|k| edge_kind_str(*k)).collect();
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
                            symbol: symbol.clone(),
                            depth: entry.depth,
                            via,
                        })
                    }
                    NodeId::QueueJob { queue_file, job } => {
                        let rel = queue_file
                            .strip_prefix(root_dir)
                            .unwrap_or(queue_file.as_path());
                        OutputNode::QueueJob(OutputQueueJob {
                            queue_file: rel.to_string_lossy().into_owned(),
                            job: job.clone(),
                            depth: entry.depth,
                            via,
                        })
                    }
                    NodeId::Module(specifier) => OutputNode::Module(OutputModule {
                        module: specifier.clone(),
                        depth: entry.depth,
                        via,
                    }),
                }
            })
            .collect(),
    }
}
