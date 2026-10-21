use std::cell::Cell;

thread_local! {
    static STATE: Cell<(bool, usize)> = const { Cell::new((false, 0)) };
}

pub(super) fn record() {
    STATE.with(|state| {
        let (legacy, visits) = state.get();
        state.set((legacy, visits + 1));
    });
}

pub(super) fn legacy() -> bool {
    STATE.with(|state| state.get().0)
}

pub(super) struct Guard((bool, usize));
impl Guard {
    pub(super) fn new(legacy: bool) -> Self {
        Self(STATE.with(|state| state.replace((legacy, 0))))
    }
    pub(super) fn visits(&self) -> usize {
        STATE.with(|state| state.get().1)
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        STATE.with(|state| state.set(self.0));
    }
}

macro_rules! record_join_binding {
    () => {
        binding_scan_count::record();
    };
}
macro_rules! legacy_join {
    ($joined:expr, $current:expr, $original:expr, $whole_sql:expr) => {
        if binding_scan_count::legacy() {
            legacy::join_with_sql($joined, $current, $original, $whole_sql);
            return;
        }
    };
}
