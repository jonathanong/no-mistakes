#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ImportTarget {
    File(PathBuf),
    /// A node builtin or external package. It cannot import a project module.
    External,
    Unresolved,
    Ambiguous,
}

pub(crate) type ImportTargetMap = FxHashMap<PathBuf, FxHashMap<String, ImportTarget>>;

pub(crate) struct ImportPass {
    pub(crate) edges: Vec<Edge>,
    pub(crate) targets: ImportTargetMap,
}

impl Default for ImportPass {
    fn default() -> Self {
        Self {
            edges: Vec::new(),
            targets: fx_map(),
        }
    }
}

impl std::ops::Deref for ImportPass {
    type Target = Vec<Edge>;

    fn deref(&self) -> &Self::Target {
        &self.edges
    }
}

pub(crate) fn merge_import_target(
    map: &mut ImportTargetMap,
    importer: &Path,
    specifier: &str,
    next: ImportTarget,
) {
    let specs = map.entry(importer.to_path_buf()).or_default();
    match specs.get(specifier) {
        Some(current) if current == &next => {}
        Some(_) => {
            specs.insert(specifier.to_string(), ImportTarget::Ambiguous);
        }
        None => {
            specs.insert(specifier.to_string(), next);
        }
    }
}

impl DepGraph {
    pub(crate) fn import_target(&self, importer: &Path, specifier: &str) -> Option<&ImportTarget> {
        self.import_targets.get(importer)?.get(specifier)
    }
}
