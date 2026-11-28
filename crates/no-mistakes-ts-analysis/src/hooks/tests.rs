use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

struct TestObserver(AtomicU64);

impl MetricsObserver for TestObserver {
    fn increment(&self, _metric: &'static str, amount: u64) {
        self.0.fetch_add(amount, Ordering::Relaxed);
    }
}

struct TestHooks {
    observer: Arc<TestObserver>,
}

impl AnalysisHooks for TestHooks {
    fn current_observer(&self) -> Option<Arc<dyn MetricsObserver>> {
        Some(Arc::clone(&self.observer) as Arc<dyn MetricsObserver>)
    }

    fn discover_visible_paths(&self, root: &Path) -> Vec<PathBuf> {
        vec![root.join("index.ts")]
    }

    fn load_workspace_from_files(
        &self,
        _root: &Path,
        _visible_paths: &[PathBuf],
    ) -> Option<WorkspaceMap> {
        Some(WorkspaceMap::default())
    }
}

#[test]
fn hooks_default_safely_when_unregistered_and_delegate_when_present() {
    assert!(current_observer_with(None).is_none());
    assert!(discover_visible_paths_with(None, Path::new("/repo")).is_empty());
    assert!(load_workspace_from_files_with(None, Path::new("/repo"), &[]).is_none());

    let observer = Arc::new(TestObserver(AtomicU64::new(0)));
    let hooks = TestHooks {
        observer: Arc::clone(&observer),
    };
    let active = current_observer_with(Some(&hooks)).expect("observer");
    active.increment("test.metric", 3);
    assert_eq!(observer.0.load(Ordering::Relaxed), 3);
    assert_eq!(
        discover_visible_paths_with(Some(&hooks), Path::new("/repo")),
        vec![PathBuf::from("/repo/index.ts")]
    );
    assert!(load_workspace_from_files_with(Some(&hooks), Path::new("/repo"), &[]).is_some());
}
