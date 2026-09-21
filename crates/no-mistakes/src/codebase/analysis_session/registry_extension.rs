use super::*;

type RegistryExtensionResult =
    Result<Arc<crate::registry_extension_query::RegistryExtensionReport>, Arc<str>>;
pub(super) type RegistryExtensionCell = Arc<OnceLock<RegistryExtensionResult>>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct RegistryExtensionKey {
    pub(super) root: PathBuf,
    pub(super) path: PathBuf,
}

impl AnalysisSession {
    /// Memoize a request-owned registry-extension report projection for one
    /// root/file pair. This is not a canonical TS fact because it is the
    /// query's rendered report, but it owns no OXC data and can therefore be
    /// reused without parsing the same source again.
    pub(crate) fn registry_extension_report(
        &self,
        root: &Path,
        path: &Path,
        build: impl FnOnce() -> anyhow::Result<crate::registry_extension_query::RegistryExtensionReport>,
    ) -> anyhow::Result<crate::registry_extension_query::RegistryExtensionReport> {
        let key = RegistryExtensionKey {
            root: normalize_path(root),
            path: normalize_path(path),
        };
        let cell = match self.registry_extension_reports.entry(key) {
            Entry::Occupied(entry) => Arc::clone(entry.get()),
            Entry::Vacant(entry) => {
                let cell = Arc::new(OnceLock::new());
                entry.insert(Arc::clone(&cell));
                cell
            }
        };
        cell.get_or_init(|| {
            build()
                .map(Arc::new)
                .map_err(|error| Arc::<str>::from(format!("{error:#}")))
        })
        .clone()
        .map(|report| (*report).clone())
        .map_err(|error| anyhow::anyhow!(error))
    }
}
