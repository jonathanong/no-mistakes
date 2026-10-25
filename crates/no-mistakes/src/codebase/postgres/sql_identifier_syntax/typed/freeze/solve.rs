use super::*;
use std::collections::VecDeque;
mod evaluate;
mod index;
use index::Index;
mod project;
use evaluate::evaluate;
const WORK_LIMIT: usize = 100_000;
impl Tracker {
    pub fn finish(mut self, sources: &Sources<'_>) -> Proof {
        if !self.enabled {
            return Proof::default();
        }
        let valid = self.clean
            && !self.global_escape
            && self.globals == self.candidates
            && !self.write_roots.iter().any(|at| self.globals.contains(at));
        if !valid {
            for node in self.nodes.values_mut() {
                if matches!(node, Node::Freeze(_)) {
                    *node = Node::Scalar;
                }
            }
        }
        self.solve(sources).unwrap_or_default()
    }
    fn solve(self, sources: &Sources<'_>) -> Option<Proof> {
        let initial = sources.initial;
        let writes = sources.writes;
        let index = Index::new(&self);
        let mut values = initial
            .iter()
            .filter(|id| !sources.producers.contains(id))
            .map(|id| (*id, Taint::executor()))
            .collect::<FxHashMap<_, _>>();
        let mut functions = sources
            .producers
            .iter()
            .map(|id| (*id, Taint::executor()))
            .collect::<FxHashMap<_, _>>();
        let mut initial_functions: VecDeque<_> = sources.producers.iter().copied().collect();
        while let Some(id) = initial_functions.pop_front() {
            for alias in index.aliases.get(&id).into_iter().flatten() {
                let value = functions[&id].clone();
                if functions.entry(*alias).or_default().join(value) {
                    initial_functions.push_back(*alias);
                }
            }
        }
        let mut facts = FxHashMap::<Span, Taint>::default();
        let mut queue: VecDeque<_> = self.order.iter().copied().collect();
        let mut work = 0;
        while let Some(at) = queue.pop_front() {
            work += 1;
            if work > WORK_LIMIT {
                return None;
            }
            let mut value = evaluate(self.nodes.get(&at)?, &facts, &values, &functions);
            if sources.data.contains(&at) || sources.candidates.contains(&at) {
                if let Some(Node::Ref(id, _)) = self.nodes.get(&at).filter(
                    |node| matches!(node, Node::Ref(id, _) if sources.producers.contains(id)),
                ) {
                    value.join(Taint::producer(*id));
                } else {
                    value.join(Taint::executor());
                }
            }
            if matches!(self.nodes.get(&at), Some(Node::Object(_))) {
                value.objects.insert((Vec::new(), at));
            }
            if !facts.entry(at).or_default().join(value) {
                continue;
            }
            queue.extend(index.parents.get(&at).into_iter().flatten().copied());
            for (id, path) in index.outputs.get(&at).into_iter().flatten() {
                let mut value = facts[&at].clone();
                for key in path {
                    value = value.read(key.as_deref());
                }
                if value.derived && writes.contains(id) {
                    value.broad = true;
                    value.paths.clear();
                }
                if values.entry(*id).or_default().join(value) {
                    queue.extend(index.bindings.get(id).into_iter().flatten().copied());
                }
            }
            for id in index.stores.get(&at).into_iter().flatten() {
                let mut value = facts[&at].clone();
                if value.present() {
                    value.broad = true;
                    value.paths.clear();
                }
                if values.entry(*id).or_default().join(value) {
                    queue.extend(index.bindings.get(id).into_iter().flatten().copied());
                }
            }
            let mut pending = VecDeque::new();
            for id in index.returns.get(&at).into_iter().flatten() {
                if functions.entry(*id).or_default().join(facts[&at].clone()) {
                    pending.push_back(*id);
                }
            }
            while let Some(id) = pending.pop_front() {
                work += 1;
                if work > WORK_LIMIT {
                    return None;
                }
                if writes.contains(&id) && functions[&id].derived {
                    let mut value = functions[&id].clone();
                    value.broad = true;
                    value.paths.clear();
                    if values.entry(id).or_default().join(value) {
                        queue.extend(index.bindings.get(&id).into_iter().flatten().copied());
                    }
                }
                queue.extend(index.invokes.get(&id).into_iter().flatten().copied());
                queue.extend(index.bindings.get(&id).into_iter().flatten().copied());
                for alias in index.aliases.get(&id).into_iter().flatten() {
                    let value = functions[&id].clone();
                    if functions.entry(*alias).or_default().join(value) {
                        pending.push_back(*alias);
                    }
                }
            }
        }
        if self.barriers.iter().any(|at| {
            facts
                .get(at)
                .is_some_and(|value| value.derived && value.present())
        }) {
            return None;
        }
        Some(self.project(values, functions, facts, sources))
    }
}
