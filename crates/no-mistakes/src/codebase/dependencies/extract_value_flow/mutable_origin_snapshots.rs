use super::*;

pub(super) enum RootSnapshots {
    Ordered(Vec<(u32, usize)>),
    Tree(BTreeMap<u32, usize>),
}
impl Default for RootSnapshots {
    fn default() -> Self {
        Self::Ordered(Vec::new())
    }
}
impl RootSnapshots {
    pub(super) fn insert(&mut self, offset: u32, selected: usize) {
        match self {
            Self::Ordered(entries) if entries.last().is_none_or(|(last, _)| *last <= offset) => {
                if let Some((_, value)) = entries.last_mut().filter(|(last, _)| *last == offset) {
                    *value = selected;
                } else {
                    entries.push((offset, selected));
                }
            }
            Self::Ordered(entries) => {
                // A genuinely earlier completion keeps the original ordered-map
                // semantics. Move the sole history; never retain a second copy.
                let mut tree: BTreeMap<_, _> = std::mem::take(entries).into_iter().collect();
                tree.insert(offset, selected);
                *self = Self::Tree(tree);
            }
            Self::Tree(tree) => {
                tree.insert(offset, selected);
            }
        }
    }
    pub(super) fn before(&self, offset: u32) -> Option<(&u32, &usize)> {
        self.selected(offset, false)
    }
    pub(super) fn at(&self, offset: u32) -> Option<(&u32, &usize)> {
        self.selected(offset, true)
    }
    fn selected(&self, offset: u32, inclusive: bool) -> Option<(&u32, &usize)> {
        self.selected_observed(offset, inclusive, |_| {})
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn at_counted(
        &self,
        offset: u32,
        work: &std::cell::Cell<[usize; 3]>,
    ) -> Option<(&u32, &usize)> {
        self.selected_observed(offset, true, |latest| {
            let mut counts = work.get();
            counts[if latest { 1 } else { 2 }] += 1;
            work.set(counts);
        })
    }
    fn selected_observed(
        &self,
        offset: u32,
        inclusive: bool,
        mut observe: impl FnMut(bool),
    ) -> Option<(&u32, &usize)> {
        match self {
            Self::Ordered(entries) => {
                // Fresh source reads usually follow the newest completion. An
                // older read still uses the exact original predecessor search.
                if let Some((at, selected)) = entries
                    .last()
                    .filter(|(at, _)| *at < offset || (inclusive && *at == offset))
                {
                    observe(true);
                    return Some((at, selected));
                }
                let end = entries.partition_point(|(at, _)| {
                    observe(false);
                    *at < offset || (inclusive && *at == offset)
                });
                end.checked_sub(1)
                    .map(|index| (&entries[index].0, &entries[index].1))
            }
            Self::Tree(tree) if inclusive => tree.range(..=offset).next_back(),
            Self::Tree(tree) => tree.range(..offset).next_back(),
        }
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn last_key_value(&self) -> Option<(&u32, &usize)> {
        match self {
            Self::Ordered(entries) => entries.last().map(|(at, value)| (at, value)),
            Self::Tree(tree) => tree.last_key_value(),
        }
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn tree_buffer(&self) -> usize {
        usize::from(matches!(self, Self::Tree(tree) if !tree.is_empty()))
    }
}
#[cfg(test)]
#[path = "mutable_origin_snapshot_tests.rs"]
mod tests;
