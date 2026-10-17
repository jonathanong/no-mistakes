#[cfg(all(test, feature = "test-instrumentation"))]
use super::*;

// Preserve every arena leaf in insertion order. A gap materializes the original
// Vec; the common consecutive history owns no buffer. The boxed fallback keeps
// this header the size of Vec<usize>, at the cost of one extra sparse allocation.
#[derive(Default)]
pub(super) struct PossibleOrigins {
    start: usize,
    end: usize,
    #[expect(
        clippy::box_collection,
        reason = "The boxed Vec keeps this struct the same size as Vec while consecutive histories allocate nothing."
    )]
    sparse: Option<Box<Vec<usize>>>,
}
impl PossibleOrigins {
    pub(super) fn push(&mut self, id: usize) {
        // Producers push arena[id] before this call, so id < arena.len() <=
        // usize::MAX. Check first even for unit callers; never wrap or drop a leaf.
        let next = id.checked_add(1).expect("arena leaf must have a successor");
        #[cfg(all(test, feature = "test-instrumentation"))]
        {
            tests::record(0, 1);
            if tests::legacy() {
                let values = self.sparse.get_or_insert_with(Default::default);
                let capacity = values.capacity();
                values.push(id);
                tests::record(4, usize::from(capacity != values.capacity()));
                return;
            }
        }
        if let Some(values) = &mut self.sparse {
            #[cfg(all(test, feature = "test-instrumentation"))]
            let capacity = values.capacity();
            values.push(id);
            #[cfg(all(test, feature = "test-instrumentation"))]
            tests::record(4, usize::from(capacity != values.capacity()));
        } else if self.start == self.end {
            self.start = id;
            self.end = next;
            #[cfg(all(test, feature = "test-instrumentation"))]
            tests::record(1, 1);
        } else if id == self.end {
            self.end = next;
            #[cfg(all(test, feature = "test-instrumentation"))]
            tests::record(2, 1);
        } else {
            let mut values: Vec<_> = (self.start..self.end).collect();
            values.push(id);
            self.sparse = Some(Box::new(values));
            #[cfg(all(test, feature = "test-instrumentation"))]
            {
                tests::record(3, 1);
                tests::record(4, 1);
            }
        }
    }
    pub(super) fn before(&self, limit: usize) -> impl Iterator<Item = usize> + '_ {
        let (range, slice) = if let Some(values) = &self.sparse {
            let end = values.partition_point(|id| *id < limit);
            #[cfg(all(test, feature = "test-instrumentation"))]
            tests::record(7, usize::from(end > 0));
            (0..0, &values[..end])
        } else {
            (self.start..self.end.min(limit).max(self.start), &[][..])
        };
        #[cfg(all(test, feature = "test-instrumentation"))]
        {
            tests::record(5, 1);
            tests::record(6, range.len() + slice.len());
        }
        #[cfg(all(test, feature = "test-instrumentation"))]
        tests::prefix(limit, range.clone().chain(slice.iter().copied()));
        range.chain(slice.iter().copied())
    }
}
const _: () = assert!(size_of::<PossibleOrigins>() == size_of::<Vec<usize>>());
const _: () =
    assert!(std::mem::align_of::<PossibleOrigins>() == std::mem::align_of::<Vec<usize>>());
// Actual default compiler type checks; feature test layout includes counters.
#[cfg(not(feature = "test-instrumentation"))]
const _: () = assert!(size_of::<super::RootStates>() == 56);
#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "mutable_origin_possible_tests.rs"]
mod tests;
