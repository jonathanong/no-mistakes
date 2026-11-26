use super::*;

thread_local! {
    static CAPTURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub(super) static CANDIDATES: std::cell::RefCell<Vec<(usize, ValueExpression)>> = const { std::cell::RefCell::new(Vec::new()) };
    pub(super) static WORK: std::cell::Cell<[usize; 5]> = const { std::cell::Cell::new([0; 5]) };
}

pub(in super::super) fn record(index: usize, amount: usize) {
    WORK.with(|work| {
        let mut values = work.get();
        values[index] += amount;
        work.set(values);
    });
}

pub(super) fn capture(items: &[(usize, ValueExpression)]) {
    if CAPTURE.with(std::cell::Cell::get) {
        CANDIDATES.with(|values| values.replace(items.to_vec()));
    }
}

#[cfg(test)]
#[path = "escaped_array_capture_tests.rs"]
pub(super) mod tests;
