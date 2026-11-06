use super::*;

pub(in super::super::super) const LIMIT: usize = 100_000;
pub(in super::super::super) const REASON: &str = "value-flow foreign receiver analysis exceeded the per-file work budget (100000 units); use a narrower analysis root to exclude vendored or minified assets, or simplify shared receiver mutations";

pub(in super::super::super) struct WorkBudget {
    remaining: usize,
    exhausted: bool,
}
impl Default for WorkBudget {
    fn default() -> Self {
        #[cfg(all(test, feature = "test-instrumentation"))]
        let remaining = tests::LIMIT_OVERRIDE
            .with(std::cell::Cell::get)
            .unwrap_or(LIMIT);
        #[cfg(not(all(test, feature = "test-instrumentation")))]
        let remaining = LIMIT;
        Self {
            remaining,
            exhausted: false,
        }
    }
}
impl WorkBudget {
    pub(in super::super::super) fn charge(&mut self, amount: usize) -> Option<()> {
        if self.exhausted {
            return None;
        }
        let Some(remaining) = self.remaining.checked_sub(amount) else {
            self.exhausted = true;
            #[cfg(all(test, feature = "test-instrumentation"))]
            tests::record(1, 1);
            return None;
        };
        self.remaining = remaining;
        #[cfg(all(test, feature = "test-instrumentation"))]
        tests::record(0, amount);
        Some(())
    }
    pub(in super::super::super) fn remaining(&self) -> usize {
        self.remaining
    }
    pub(in super::super::super) fn exhausted(&self) -> bool {
        self.exhausted
    }
    pub(in super::super::super) fn output(&mut self) -> Output<'_> {
        Output {
            budget: self,
            values: Vec::new(),
        }
    }
}
pub(in super::super::super) struct Output<'a> {
    budget: &'a mut WorkBudget,
    values: Vec<nested::ReceiverWrite>,
}
impl Extend<nested::ReceiverWrite> for Output<'_> {
    fn extend<I: IntoIterator<Item = nested::ReceiverWrite>>(&mut self, values: I) {
        for value in values {
            if self
                .budget
                .charge(1 + value.0 .1.len() + value.1.len())
                .is_some()
            {
                self.values.push(value);
            }
        }
    }
}
impl Output<'_> {
    pub(in super::super::super) fn finish(self) -> Option<Vec<nested::ReceiverWrite>> {
        (!self.budget.exhausted).then_some(self.values)
    }
}

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "foreign_work_budget_tests.rs"]
pub(super) mod tests;
