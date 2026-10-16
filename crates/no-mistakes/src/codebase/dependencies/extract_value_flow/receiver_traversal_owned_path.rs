use super::*;
use std::borrow::Cow;

#[cfg(all(test, feature = "test-instrumentation"))]
thread_local! {
    pub(super) static OLD_ENTRY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub(super) static MEMBER_CLONES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(all(test, feature = "test-instrumentation"))]
pub(super) fn record_member_clone(member: Option<&Option<String>>) {
    MEMBER_CLONES
        .with(|count| count.set(count.get() + usize::from(member.is_some_and(Option::is_some))));
}

const _: () = assert!(std::mem::size_of::<TraversalState>() <= 144);

impl TraversalState {
    pub(in super::super) fn enter_owned_path<'a>(
        &'a mut self,
        key: &Key,
        mut path: ValueMutationPath,
        reads: Vec<nested::ReceiverRead>,
    ) -> Option<OwnedEntry<'a>> {
        #[cfg(all(test, feature = "test-instrumentation"))]
        if OLD_ENTRY.with(std::cell::Cell::get) {
            return self
                .enter(key, &path, reads)
                .map(|reads| (reads, Cow::Owned(path)));
        }
        let owns_first = matches!(self, Self::Empty) && path.len() <= 1 && reads.len() <= 1;
        if owns_first {
            // The popped work item no longer needs its owned member. Keep that
            // allocation in the visited state and borrow it for this iteration.
            *self = Self::FirstInline {
                state: (key.clone(), path.pop()),
                read: reads.first().copied(),
                active: true,
            };
        }
        match (self, owns_first) {
            (Self::FirstInline { state, .. }, true) => {
                Some((reads, Cow::Borrowed(state.1.as_slice())))
            }
            (state, _) => state
                .enter(key, &path, reads)
                .map(|reads| (reads, Cow::Owned(path))),
        }
    }
}

#[cfg(test)]
#[path = "receiver_traversal_owned_path_tests.rs"]
mod tests;
