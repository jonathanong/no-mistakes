use crate::codebase::check_facts::sql_source_prerequisites::FailedSqlSources;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct State {
    failed: Arc<FailedSqlSources>,
    consulted: FailedSqlSources,
    modules: BTreeMap<PathBuf, FailedSqlSources>,
    active: Vec<PathBuf>,
}

// Semantic snapshots cannot erase inputs already consulted by this root.
#[derive(Clone, Default)]
pub(in crate::codebase::postgres::query_annotation) struct SourceDependencies(
    Option<Arc<Mutex<State>>>,
);

impl SourceDependencies {
    pub fn new(failed: Arc<FailedSqlSources>) -> Self {
        if failed.is_empty() {
            return Self::default();
        }
        Self(Some(Arc::new(Mutex::new(State {
            failed,
            ..Default::default()
        }))))
    }

    pub fn consult(&self, path: &Path) {
        let Some(state) = &self.0 else { return };
        let mut state = state.lock().unwrap();
        if state.failed.contains(path) {
            let paths = [path.to_path_buf()].into_iter().collect();
            state.record(&paths);
        }
    }

    pub fn begin(&self, path: &Path) {
        if let Some(state) = &self.0 {
            state.lock().unwrap().active.push(path.to_path_buf());
        }
    }

    pub fn finish(&self, path: &Path) {
        if let Some(state) = &self.0 {
            let active = state.lock().unwrap().active.pop();
            debug_assert_eq!(active.as_deref(), Some(path));
        }
    }

    pub fn replay(&self, path: &Path) {
        let Some(state) = &self.0 else { return };
        let mut state = state.lock().unwrap();
        if let Some(paths) = state.modules.get(path).cloned() {
            state.record(&paths);
        }
    }

    pub fn failures(&self) -> FailedSqlSources {
        self.0
            .as_ref()
            .map(|state| state.lock().unwrap().consulted.clone())
            .unwrap_or_default()
    }
}

impl State {
    fn record(&mut self, paths: &FailedSqlSources) {
        self.consulted.extend(paths.iter().cloned());
        for path in &self.active {
            self.modules
                .entry(path.clone())
                .or_default()
                .extend(paths.iter().cloned());
        }
    }
}

#[cfg(test)]
mod tests;
