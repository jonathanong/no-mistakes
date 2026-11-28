use super::{FxHashSet, QueryUses};

impl QueryUses {
    pub(super) fn resolve_recursive_demand(&mut self) -> Vec<usize> {
        let mut reachable = FxHashSet::default();
        loop {
            let previous = reachable.len();
            for (index, cte) in self.definitions.iter_mut().enumerate() {
                if (cte.forced || self.names.contains(&cte.name)) && reachable.insert(index) {
                    self.names.extend(std::mem::take(&mut cte.names));
                }
            }
            if reachable.len() == previous {
                break;
            }
        }
        for cte in &self.definitions {
            self.names.remove(&cte.name);
        }
        let mut reachable: Vec<_> = reachable.into_iter().collect();
        reachable.sort_unstable_by(|left, right| right.cmp(left));
        reachable
    }
}
