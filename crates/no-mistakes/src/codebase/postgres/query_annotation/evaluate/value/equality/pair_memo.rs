use crate::fx::FxHashSet;
type Pair = (usize, usize);
const INLINE: usize = 8;

// Shallow comparisons avoid heap allocation; larger DAGs retain every completed
// pair in a comparison-local map. Addresses stay live for the comparison.
#[derive(Default)]
pub(super) struct Memo {
    small: [Pair; INLINE],
    len: usize,
    large: FxHashSet<Pair>,
}
impl Memo {
    pub(super) fn contains(&self, pair: Pair) -> bool {
        self.small[..self.len].contains(&pair) || self.large.contains(&pair)
    }
    pub(super) fn insert(&mut self, pair: Pair) {
        if self.len < INLINE {
            self.small[self.len] = pair;
            self.len += 1;
        } else {
            self.large.insert(pair);
        }
    }
}

#[cfg(test)]
mod tests;
