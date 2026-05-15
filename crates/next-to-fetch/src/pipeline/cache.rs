use crate::report::types::FetchOccurrence;
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) struct Cache {
    pub(crate) files: HashMap<(PathBuf, bool, bool), CachedFile>,
    pub(crate) imports: HashMap<PathBuf, Vec<PathBuf>>,
}

#[derive(Clone)]
pub(crate) struct CachedFile {
    pub(crate) is_client: bool,
    pub(crate) fetches: Vec<FetchOccurrence>,
}
