use rustc_hash::{FxHashMap, FxHashSet};
use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct CollectedExcludes {
    pub(crate) default_export: Vec<ExcludeArray>,
    pub(crate) test_literals: Vec<ExcludeArray>,
    pub(crate) exports: FxHashMap<String, FxHashSet<String>>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub(crate) struct SpreadImport {
    pub(crate) specifier: String,
    pub(crate) name: String,
}

pub(crate) type ResolvedSpreads = FxHashMap<PathBuf, FxHashMap<SpreadImport, FxHashSet<String>>>;

pub(crate) struct ExcludeArray {
    pub(crate) line: usize,
    pub(crate) globs: FxHashSet<String>,
    pub(crate) spreads_config_defaults: bool,
    pub(crate) imports: Vec<SpreadImport>,
    pub(crate) unknown: bool,
}
