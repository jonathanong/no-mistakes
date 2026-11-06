use super::*;
use mutation_projection::{PendingMutation, PendingQueue};

type References<R> = reference_output::ReferenceOutput<nested::ReceiverWrite<R>>;
pub(in super::super) struct ProjectionOutput<R> {
    output: References<R>,
    remaining: Option<usize>,
    used: usize,
    exceeded: bool,
}
impl<R> Extend<nested::ReceiverWrite<R>> for ProjectionOutput<R> {
    fn extend<I: IntoIterator<Item = nested::ReceiverWrite<R>>>(&mut self, values: I) {
        let Some(mut remaining) = self.remaining else {
            self.output.extend(values);
            return;
        };
        for value in values {
            let amount = 1 + value.0 .1.len() + value.1.len();
            if let Some(next) = remaining.checked_sub(amount).filter(|_| !self.exceeded) {
                remaining = next;
                self.used += amount;
                self.output.extend(std::iter::once(value));
            } else {
                self.exceeded = true;
                break;
            }
        }
        self.remaining = Some(remaining);
    }
}
impl<R> nested::ReceiverOutput<R> for ProjectionOutput<R> {
    fn visit(&mut self) -> bool {
        let Some(remaining) = self.remaining else {
            return true;
        };
        if let Some(next) = remaining.checked_sub(1).filter(|_| !self.exceeded) {
            self.remaining = Some(next);
            self.used += 1;
            true
        } else {
            self.exceeded = true;
            false
        }
    }
    fn active(&self) -> bool {
        !self.exceeded
    }
}
impl ImportCollector {
    pub(in super::super) fn charge_foreign_link_work(
        &mut self,
        key: &(usize, String),
        width: usize,
    ) -> Option<()> {
        if self.remaining_foreign_work().is_none() {
            return Some(());
        }
        let count = self
            .value_reference_links
            .as_ref()
            .and_then(|links| links.get(&key.0))
            .and_then(|names| names.get(key.1.as_str()))
            .map_or(0, Vec::len);
        self.charge_foreign_work(count.saturating_mul(1 + width))
    }

    pub(in super::super) fn bounded_receiver_output<R>(&self) -> ProjectionOutput<R> {
        ProjectionOutput {
            output: Default::default(),
            remaining: self.remaining_foreign_work(),
            used: 0,
            exceeded: false,
        }
    }
    pub(in super::super) fn append_bounded_mutation<R: nested::RootCoordinate>(
        &mut self,
        pending: &mut PendingQueue<PendingMutation<R>>,
        exit_position: usize,
        key: (usize, String),
        root_read: R,
        assigned_exit: bool,
        references: ProjectionOutput<R>,
    ) -> bool {
        let amount = if references.exceeded {
            usize::MAX
        } else {
            references.used
        };
        if self.charge_foreign_work(amount).is_none() {
            return false;
        }
        #[cfg(feature = "test-instrumentation")]
        references.output.record_counts(&mut self.value_flow);
        mutation_projection::pending_work::append(
            pending,
            exit_position,
            key,
            root_read,
            assigned_exit,
            references.output,
        );
        true
    }
}

#[cfg(test)]
#[path = "foreign_projection_budget_tests.rs"]
mod tests;
