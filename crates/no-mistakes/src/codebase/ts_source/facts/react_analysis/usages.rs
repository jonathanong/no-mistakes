use std::path::PathBuf;

/// A resolved JSX render site before it is filtered against a target component.
#[derive(Clone)]
pub(crate) struct RawCallsite {
    pub(crate) resolved_path: PathBuf,
    pub(crate) exported_name: String,
    pub(crate) line: usize,
    pub(crate) props: Vec<String>,
    pub(crate) has_spread: bool,
}

#[derive(Clone)]
pub(crate) struct UsageFileFacts {
    pub(crate) imports: Vec<(PathBuf, String)>,
    pub(crate) callsites: Vec<RawCallsite>,
}
