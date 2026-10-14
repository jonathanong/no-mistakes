use super::*;

#[derive(Default)]
pub(in super::super) struct ReceiverHistories {
    names: FxHashMap<usize, FxHashMap<String, usize>>,
    components: Vec<Component>,
    histories: FxHashMap<usize, mutation_history::ValueMutationHistory>,
    proofs: FxHashMap<usize, bool>,
    #[cfg(feature = "test-instrumentation")]
    pub(super) links: usize,
    #[cfg(feature = "test-instrumentation")]
    pub(super) root_steps: std::cell::Cell<usize>,
}

struct Component {
    parent: usize,
    size: usize,
}

impl ReceiverHistories {
    fn intern(&mut self, scope: usize, name: &str) -> usize {
        if let Some(id) = self.names.get(&scope).and_then(|names| names.get(name)) {
            return *id;
        }
        let id = self.components.len();
        self.components.push(Component {
            parent: id,
            size: 1,
        });
        self.names
            .entry(scope)
            .or_default()
            .insert(name.to_string(), id);
        id
    }

    fn root(&self, mut id: usize) -> usize {
        loop {
            #[cfg(feature = "test-instrumentation")]
            self.root_steps.set(self.root_steps.get() + 1);
            let parent = self.components[id].parent;
            if parent == id {
                return id;
            }
            id = parent;
        }
    }

    pub(super) fn link(&mut self, left: (usize, &str), right: (usize, &str)) {
        #[cfg(feature = "test-instrumentation")]
        {
            self.links += 1;
        }
        let left = self.intern(left.0, left.1);
        let right = self.intern(right.0, right.1);
        let (mut left, mut right) = (self.root(left), self.root(right));
        if left == right {
            return;
        }
        // Union by size bounds every later lookup without per-write graph walks.
        if self.components[left].size < self.components[right].size {
            std::mem::swap(&mut left, &mut right);
        }
        let proof = match (self.proofs.remove(&left), self.proofs.remove(&right)) {
            (Some(left), Some(right)) => Some(left && right),
            (Some(proof), None) => Some(proof && !self.histories.contains_key(&right)),
            (None, Some(proof)) => Some(proof && !self.histories.contains_key(&left)),
            (None, None) => None,
        };
        if let Some(proof) = proof {
            self.proofs.insert(left, proof);
        }
        self.components[right].parent = left;
        self.components[left].size += self.components[right].size;
        if let Some(history) = self.histories.remove(&right) {
            self.histories.entry(left).or_default().merge(history);
        }
    }

    pub(super) fn require_reference_proof(&mut self, scope: usize, name: &str) {
        let id = self.intern(scope, name);
        let root = self.root(id);
        let complete = !self.histories.contains_key(&root);
        self.proofs.entry(root).or_insert(complete);
    }

    pub(super) fn is_empty(&self) -> bool {
        self.histories.is_empty()
    }

    pub(super) fn contains_history(&self, scope: usize, name: &str) -> bool {
        self.names
            .get(&scope)
            .and_then(|names| names.get(name))
            .is_some_and(|id| self.histories.contains_key(&self.root(*id)))
    }

    pub(super) fn existing_history_mut(
        &mut self,
        scope: usize,
        name: &str,
    ) -> Option<&mut mutation_history::ValueMutationHistory> {
        let id = *self.names.get(&scope)?.get(name)?;
        let root = self.root(id);
        self.histories.get_mut(&root)
    }

    pub(super) fn history_mut(
        &mut self,
        key: &(usize, String),
    ) -> (&mut mutation_history::ValueMutationHistory, bool) {
        let id = self.intern(key.0, &key.1);
        let root = self.root(id);
        let proof = self.proofs.get(&root) == Some(&true);
        (self.histories.entry(root).or_default(), proof)
    }

    pub(super) fn history(
        &self,
        key: &(usize, String),
    ) -> Option<(&mutation_history::ValueMutationHistory, bool)> {
        let id = self.names.get(&key.0)?.get(key.1.as_str())?;
        let root = self.root(*id);
        Some((
            self.histories.get(&root)?,
            self.proofs.get(&root) == Some(&true),
        ))
    }

    pub(super) fn finish(
        &mut self,
    ) -> Vec<(Vec<(usize, String)>, mutation_history::ValueMutationHistory)> {
        let mut members: FxHashMap<usize, Vec<(usize, String)>> = fx_map();
        for (scope, names) in std::mem::take(&mut self.names) {
            for (name, id) in names {
                let root = self.root(id);
                if self.histories.contains_key(&root) {
                    members.entry(root).or_default().push((scope, name));
                }
            }
        }
        members
            .into_iter()
            .map(|(root, names)| {
                (
                    names,
                    self.histories.remove(&root).expect("mutated component"),
                )
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "receiver_histories_tests.rs"]
mod tests;
