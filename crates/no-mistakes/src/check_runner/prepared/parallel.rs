/// Book one overlapping `check` preparation branch.
///
/// `with_observer` resets the thread to serial so a stolen task cannot inherit
/// an unrelated marker. These branches run together, so the reset is then
/// marked parallel. Nested traces stay non-additive with the prepare wall.
pub(super) fn branch<T>(
    observer: Option<std::sync::Arc<no_mistakes::diagnostics::InvocationObserver>>,
    label: &'static str,
    operation: impl FnOnce() -> T,
) -> T {
    no_mistakes::diagnostics::with_observer(observer, || {
        no_mistakes::diagnostics::with_timing_kind(
            no_mistakes::diagnostics::TimingKind::Parallel,
            || {
                if let Some(observer) = no_mistakes::diagnostics::current() {
                    observer.increment(label, 1);
                }
                no_mistakes::perf_trace::trace(label, operation)
            },
        )
    })
}
