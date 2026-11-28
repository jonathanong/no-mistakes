use std::path::PathBuf;

/// A resolved JSX render site before it is filtered against a target component.
#[derive(Clone)]
pub struct RawCallsite {
    pub resolved_path: PathBuf,
    pub exported_name: String,
    pub line: usize,
    pub props: Vec<String>,
    pub has_spread: bool,
}

#[derive(Clone)]
pub struct UsageFileFacts {
    pub imports: Vec<(PathBuf, String)>,
    pub callsites: Vec<RawCallsite>,
}
