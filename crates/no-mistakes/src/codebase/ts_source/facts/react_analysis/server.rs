use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Framework {
    ApiServer,
    Express,
    Fastify,
    Hono,
    KoaPathMatch,
    KoaRouter,
    Nestjs,
    Heuristic,
    Remix,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Binding {
    pub framework: Framework,
    pub prefixes: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct RouteSite {
    pub file: PathBuf,
    pub line: usize,
    pub binding: String,
    pub method: String,
    pub raw_path: String,
    pub path: String,
    pub query_params: Vec<String>,
    pub framework: Framework,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct MountSite {
    pub parent: String,
    pub child: String,
    pub prefix: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ImportBinding {
    pub local: String,
    pub imported: String,
    pub source: String,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct FileFacts {
    pub bindings: HashMap<String, Binding>,
    pub exports: HashMap<String, String>,
    pub imports: Vec<ImportBinding>,
    pub routes: Vec<RouteSite>,
    pub client_calls: Vec<String>,
    pub mounts: Vec<MountSite>,
    pub diagnostics: Vec<(usize, String)>,
}
