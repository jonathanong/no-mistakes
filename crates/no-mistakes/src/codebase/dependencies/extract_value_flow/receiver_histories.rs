use super::*;

#[derive(Default)]
pub(in super::super) struct ReceiverHistories {
    names: FxHashMap<usize, FxHashMap<String, usize>>,
    components: Vec<Component>,
    histories: FxHashMap<usize, mutation_history::ValueMutationHistory>,
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
        self.components[right].parent = left;
        self.components[left].size += self.components[right].size;
        if let Some(history) = self.histories.remove(&right) {
            self.histories.entry(left).or_default().merge(history);
        }
    }

    pub(super) fn history_mut(
        &mut self,
        key: &(usize, String),
    ) -> &mut mutation_history::ValueMutationHistory {
        let id = self.intern(key.0, &key.1);
        let root = self.root(id);
        self.histories.entry(root).or_default()
    }

    pub(super) fn history(
        &self,
        key: &(usize, String),
    ) -> Option<&mutation_history::ValueMutationHistory> {
        let id = self.names.get(&key.0)?.get(key.1.as_str())?;
        self.histories.get(&self.root(*id))
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
