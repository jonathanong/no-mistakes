use super::{InvocationObserver, TimingKind};
use std::cell::RefCell;
use std::sync::Arc;
use std::time::{Duration, Instant};

thread_local! {
    static ACTIVE: RefCell<Vec<Arc<InvocationObserver>>> = const { RefCell::new(Vec::new()) };
    static TIMING_CONTEXT: RefCell<Vec<TimingKind>> = const { RefCell::new(Vec::new()) };
}

/// Return the observer scoped to this execution thread. Rayon entrypoints must
/// propagate the invocation observer explicitly with [`with_observer`]; an
/// unrelated concurrent request can never observe another thread's state.
pub fn current() -> Option<Arc<InvocationObserver>> {
    ACTIVE.with(|active| active.borrow().last().cloned())
}

pub struct InvocationGuard;

impl InvocationGuard {
    pub fn install(observer: Arc<InvocationObserver>) -> Self {
        ACTIVE.with(|active| active.borrow_mut().push(observer));
        Self
    }
}

impl Drop for InvocationGuard {
    fn drop(&mut self) {
        ACTIVE.with(|active| {
            active
                .borrow_mut()
                .pop()
                .expect("diagnostics observer scope must be active");
        });
    }
}

/// Run work with an invocation observer scoped to the current thread. This is
/// the boundary helper for Rayon tasks and other explicitly spawned work.
pub fn with_observer<T>(
    observer: Option<Arc<InvocationObserver>>,
    operation: impl FnOnce() -> T,
) -> T {
    let Some(observer) = observer else {
        return operation();
    };
    let _guard = InvocationGuard::install(observer);
    operation()
}

pub fn current_timing_kind() -> TimingKind {
    TIMING_CONTEXT.with(|context| {
        context
            .borrow()
            .last()
            .copied()
            .unwrap_or(TimingKind::Serial)
    })
}

pub(super) fn with_timing_kind<T>(kind: TimingKind, operation: impl FnOnce() -> T) -> T {
    TIMING_CONTEXT.with(|context| context.borrow_mut().push(kind));
    struct TimingGuard;
    impl Drop for TimingGuard {
        fn drop(&mut self) {
            TIMING_CONTEXT.with(|context| {
                context
                    .borrow_mut()
                    .pop()
                    .expect("diagnostics timing scope must be active");
            });
        }
    }
    let _guard = TimingGuard;
    operation()
}

/// Adapts legacy Rust argument structs that expose a `timings` boolean. CLI
/// calls already have a root observer, so the adapter becomes a no-op there;
/// direct Rust calls get the same diagnostics without changing signatures.
pub struct LegacyDiagnosticsGuard {
    pub(super) observer: Option<Arc<InvocationObserver>>,
    _guard: Option<InvocationGuard>,
}

impl LegacyDiagnosticsGuard {
    pub fn new(timings: bool, verbose: bool) -> Self {
        if !timings && !verbose || current().is_some() {
            return Self {
                observer: None,
                _guard: None,
            };
        }
        let observer = InvocationObserver::new(verbose);
        let guard = InvocationGuard::install(Arc::clone(&observer));
        Self {
            observer: Some(observer),
            _guard: Some(guard),
        }
    }
}

impl Drop for LegacyDiagnosticsGuard {
    fn drop(&mut self) {
        if let Some(observer) = &self.observer {
            observer.render_stderr();
        }
    }
}

/// Measure only when an observer exists. The disabled branch calls the
/// operation directly and performs no `Instant::now()` or `elapsed()` calls.
pub fn measure_if_enabled<T>(
    label: &'static str,
    kind: TimingKind,
    operation: impl FnOnce() -> T,
) -> (T, Duration) {
    match current() {
        Some(observer) => observer.measure(label, kind, operation),
        None => (operation(), Duration::ZERO),
    }
}

pub(super) fn measure_optional<T>(
    observer: Option<&InvocationObserver>,
    now: impl Fn() -> Instant,
    operation: impl FnOnce() -> T,
) -> (T, Duration) {
    let Some(_observer) = observer else {
        return (operation(), Duration::ZERO);
    };
    let started = now();
    let result = operation();
    (result, now().duration_since(started))
}
