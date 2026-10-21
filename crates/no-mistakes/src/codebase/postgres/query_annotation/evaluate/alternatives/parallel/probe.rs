//! Test-only rendezvous proves that different Rayon workers execute fork tasks.
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::sync::{Arc, Barrier, Mutex};
thread_local! { static CURRENT: RefCell<Option<Arc<Probe>>> = const { RefCell::new(None) }; }
pub(super) struct Probe {
    pub workers: Mutex<Vec<usize>>,
    barrier: Option<Barrier>,
}
impl Probe {
    pub fn new(parallel: bool) -> Arc<Self> {
        Arc::new(Self {
            workers: Mutex::new(Vec::new()),
            barrier: parallel.then(|| Barrier::new(2)),
        })
    }
    pub fn enter(&self) {
        self.workers
            .lock()
            .unwrap()
            .push(rayon::current_thread_index().unwrap());
        if let Some(barrier) = &self.barrier {
            barrier.wait();
        }
    }
    pub fn assert(&self, parallel: bool) {
        let workers = self.workers.lock().unwrap();
        assert_eq!(
            workers.len(),
            if parallel { 2 } else { 0 },
            "jobs1 bypasses forks; parallel jobs execute both arms"
        );
        assert_eq!(
            workers.iter().copied().collect::<BTreeSet<_>>().len(),
            if parallel { 2 } else { 0 }
        );
    }
}
pub(super) fn current() -> Option<Arc<Probe>> {
    CURRENT.with_borrow(Clone::clone)
}
pub(super) fn installed<T>(probe: Arc<Probe>, run: impl FnOnce() -> T) -> T {
    CURRENT.with_borrow_mut(|slot| assert!(slot.replace(probe).is_none()));
    let result = run();
    CURRENT.with_borrow_mut(|slot| *slot = None);
    result
}
