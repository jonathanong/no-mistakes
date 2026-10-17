use super::progress::ActiveSpanGuard;
use super::{InvocationObserver, TimingKind};
use std::cell::{Cell, RefCell};
use std::sync::Arc;
use std::time::{Duration, Instant};

thread_local! {
    static ACTIVE: RefCell<Vec<Option<Arc<InvocationObserver>>>> = const { RefCell::new(Vec::new()) };
    static WORK_ENABLED: Cell<bool> = const { Cell::new(false) };
    static TIMING_CONTEXT: RefCell<Vec<TimingKind>> = const { RefCell::new(Vec::new()) };
}

/// Return the observer scoped to this execution thread. Rayon entrypoints must
/// propagate the invocation observer explicitly with [`with_observer`]; an
/// unrelated concurrent request can never observe another thread's state.
pub fn current() -> Option<Arc<InvocationObserver>> {
    ACTIVE.with(|active| active.borrow().last().cloned().flatten())
}

/// Borrow the thread-local observer without cloning its `Arc`.
///
/// The closure must not re-enter this thread's observer stack (`current`,
/// `with_current`, or `with_observer`); that stack borrow is held for the call.
pub fn with_current<R>(operation: impl FnOnce(&InvocationObserver) -> R) -> Option<R> {
    ACTIVE.with(|active| {
        let active = active.borrow();
        active
            .last()
            .and_then(|observer| observer.as_deref())
            .map(operation)
    })
}

/// True only when this thread's active observer records work counters.
/// Hot loops use this as the diagnostics-off empty check.
#[inline]
pub(crate) fn work_enabled() -> bool {
    WORK_ENABLED.with(Cell::get)
}

fn push_scope(observer: Option<Arc<InvocationObserver>>) {
    ACTIVE.with(|active| active.borrow_mut().push(observer));
    sync_work_enabled();
}

fn pop_scope() {
    ACTIVE.with(|active| {
        active
            .borrow_mut()
            .pop()
            .expect("diagnostics observer scope must be active");
    });
    sync_work_enabled();
}

fn sync_work_enabled() {
    let enabled = ACTIVE.with(|active| {
        active
            .borrow()
            .last()
            .and_then(|observer| observer.as_ref())
            .is_some_and(|observer| observer.verbose())
    });
    WORK_ENABLED.with(|flag| flag.set(enabled));
}

pub struct InvocationGuard;

impl InvocationGuard {
    pub fn install(observer: Arc<InvocationObserver>) -> Self {
        push_scope(Some(observer));
        Self
    }
}

impl Drop for InvocationGuard {
    fn drop(&mut self) {
        pop_scope();
    }
}

/// Run work with an invocation observer scoped to the current thread. This is
/// the boundary helper for Rayon tasks and other explicitly spawned work.
///
/// `None` pushes a disabled scope so an uninstrumented task cannot record
/// into another request. A later `Some(observer)` on the same stack overrides
/// that disabled scope. The timing context is reset to `Serial` so a stolen
/// task does not inherit an unrelated `Parallel` marker.
pub fn with_observer<T>(
    observer: Option<Arc<InvocationObserver>>,
    operation: impl FnOnce() -> T,
) -> T {
    match observer {
        Some(observer) => {
            let _guard = InvocationGuard::install(observer);
            with_timing_kind(TimingKind::Serial, operation)
        }
        None => {
            push_scope(None);
            struct DisabledGuard;
            impl Drop for DisabledGuard {
                fn drop(&mut self) {
                    pop_scope();
                }
            }
            let _disabled = DisabledGuard;
            with_timing_kind(TimingKind::Serial, operation)
        }
    }
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

pub fn with_timing_kind<T>(kind: TimingKind, operation: impl FnOnce() -> T) -> T {
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
    let observer = current();
    measure_optional(observer.as_deref(), label, kind, Instant::now, operation)
}

pub(super) fn measure_optional<T>(
    observer: Option<&InvocationObserver>,
    label: &str,
    kind: TimingKind,
    now: impl Fn() -> Instant,
    operation: impl FnOnce() -> T,
) -> (T, Duration) {
    let Some(observer) = observer else {
        return (operation(), Duration::ZERO);
    };
    let started = now();
    let _span = ActiveSpanGuard::start(observer, label, kind, started);
    let result = with_timing_kind(kind, operation);
    let duration = now().duration_since(started);
    observer.record_duration(label, duration, kind);
    (result, duration)
}
