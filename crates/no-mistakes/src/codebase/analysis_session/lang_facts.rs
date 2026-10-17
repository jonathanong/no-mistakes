use crate::codebase::lang_frontends::{collect_all_lang_facts, LangFrontendConfig};
use crate::codebase::ts_source::SourceStore;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use rustc_hash::FxBuildHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use super::AnalysisSession;

type Facts = crate::codebase::lang_frontends::CollectedLangFacts;
type FactsCell = Arc<OnceLock<Arc<Facts>>>;

/// Request cache for language-frontend extraction.
///
/// Callers in one session can pass different package config, file lists, or
/// source stores (graph discovery drops skipped directories; queue and routes
/// use the visible snapshot). The key is those inputs. A shared `Arc` is the
/// successful extraction; per-file read failures stay inside that map.
pub(super) struct LangFactsCache {
    entries: DashMap<LangFactsKey, FactsCell, FxBuildHasher>,
}

struct LangFactsKey {
    root: PathBuf,
    config: LangFrontendConfig,
    files: Arc<Vec<PathBuf>>,
    sources: Arc<SourceStore>,
}

impl PartialEq for LangFactsKey {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.sources, &other.sources)
            && self.root == other.root
            && self.config == other.config
            && (Arc::ptr_eq(&self.files, &other.files)
                || self.files.as_slice() == other.files.as_slice())
    }
}

impl Eq for LangFactsKey {}

impl Hash for LangFactsKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.root.hash(state);
        self.config.hash(state);
        self.files.as_slice().hash(state);
        Arc::as_ptr(&self.sources).hash(state);
    }
}

impl LangFactsCache {
    pub(super) fn new() -> Self {
        Self {
            entries: DashMap::with_hasher(FxBuildHasher),
        }
    }

    fn get_or_compute(
        &self,
        session: &AnalysisSession,
        root: &Path,
        files: &Arc<Vec<PathBuf>>,
        config: &LangFrontendConfig,
        sources: &Arc<SourceStore>,
    ) -> Arc<Facts> {
        let key = LangFactsKey {
            root: root.to_path_buf(),
            config: config.clone(),
            files: Arc::clone(files),
            sources: Arc::clone(sources),
        };
        let cell = match self.entries.entry(key) {
            Entry::Occupied(entry) => Arc::clone(entry.get()),
            Entry::Vacant(entry) => {
                let cell = Arc::new(OnceLock::new());
                entry.insert(Arc::clone(&cell));
                cell
            }
        };
        Arc::clone(cell.get_or_init(|| compute(session, root, files, config, sources)))
    }
}

impl AnalysisSession {
    pub(crate) fn lang_frontend_facts(
        &self,
        root: &Path,
        files: &Arc<Vec<PathBuf>>,
        config: &LangFrontendConfig,
        sources: &Arc<SourceStore>,
    ) -> Arc<Facts> {
        self.lang_facts
            .get_or_compute(self, root, files, config, sources)
    }
}

fn compute(
    session: &AnalysisSession,
    root: &Path,
    files: &[PathBuf],
    config: &LangFrontendConfig,
    sources: &SourceStore,
) -> Arc<Facts> {
    let facts = collect_all_lang_facts(root, files, config, sources);
    let scanned = facts.python.scanned_files
        + facts.go.scanned_files
        + facts.rust.scanned_files
        + facts.ruby.scanned_files
        + facts.php.scanned_files
        + facts.java.scanned_files
        + facts.kotlin.scanned_files
        + facts.elixir.scanned_files
        + facts.dart.scanned_files;
    session.record_work("lang_frontends.file_extractions", scanned as u64);
    Arc::new(facts)
}
