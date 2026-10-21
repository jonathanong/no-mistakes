use super::*;

#[derive(Default)]
pub(super) struct Candidates {
    pub(super) items: Vec<(usize, ValueExpression)>,
    buckets: FxHashMap<u64, Vec<usize>>,
}
impl Candidates {
    pub(super) fn insert(&mut self, item: (usize, ValueExpression)) {
        let key = super::hash::fingerprint(&item);
        let bucket = self.buckets.entry(key).or_default();
        if !bucket.iter().any(|index| self.items[*index] == item) {
            bucket.push(self.items.len());
            self.items.push(item);
        }
    }
    pub(super) fn contains(&self, item: &(usize, ValueExpression)) -> bool {
        self.buckets
            .get(&super::hash::fingerprint(item))
            .is_some_and(|bucket| bucket.iter().any(|index| self.items[*index] == *item))
    }
}

#[cfg(test)]
#[path = "escaped_array_candidate_tests.rs"]
mod tests;
