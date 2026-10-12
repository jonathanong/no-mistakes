use super::{ActiveSpan, InvocationObserver, TimingKind};
use std::cell::RefCell;
use std::time::Instant;

thread_local! {
    // The observer identity prevents a nested unrelated request inheriting a phase.
    static SPANS: RefCell<Vec<(usize, u64, String)>> = const { RefCell::new(Vec::new()) };
}

pub(in crate::diagnostics) struct ActiveSpanGuard<'a> {
    observer: &'a InvocationObserver,
    id: Option<u64>,
}

impl<'a> ActiveSpanGuard<'a> {
    pub(in crate::diagnostics) fn start(
        observer: &'a InvocationObserver,
        label: &str,
        kind: TimingKind,
        started: Instant,
    ) -> Self {
        let id = observer.progress.as_ref().map(|progress| {
            let identity = std::ptr::from_ref(observer) as usize;
            let mut state = progress
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.started += 1;
            let id = state.started;
            let mut phase = SPANS.with(|spans| {
                let mut spans = spans.borrow_mut();
                let mut phase = spans
                    .iter()
                    .filter(|(owner, _, _)| *owner == identity)
                    .map(|(_, _, label)| label.clone())
                    .collect::<Vec<_>>();
                phase.push(label.to_string());
                spans.push((identity, id, label.to_string()));
                phase
            });
            // Rayon tasks carry the request observer, not another thread's TLS
            // stack. The explicit dispatch span remains their common ancestor.
            if phase
                .first()
                .is_none_or(|label| label != "invocation.dispatch")
                && state
                    .active
                    .values()
                    .any(|span| span.phase == ["invocation.dispatch"])
            {
                phase.insert(0, "invocation.dispatch".to_string());
            }
            state.active.insert(
                id,
                ActiveSpan {
                    phase,
                    started,
                    parallel: kind == TimingKind::Parallel,
                },
            );
            id
        });
        Self { observer, id }
    }
}

impl Drop for ActiveSpanGuard<'_> {
    fn drop(&mut self) {
        let Some(id) = self.id else {
            return;
        };
        let mut state = self
            .observer
            .progress
            .as_ref()
            .unwrap()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.active.remove(&id);
        state.completed += 1;
        SPANS.with(|spans| {
            spans.borrow_mut().pop();
        });
    }
}
