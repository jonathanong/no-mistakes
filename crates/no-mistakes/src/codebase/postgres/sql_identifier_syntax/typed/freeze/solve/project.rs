use super::*;
mod methods;
mod uses;

impl Tracker {
    pub(super) fn project(
        self,
        values: FxHashMap<usize, Taint>,
        functions: FxHashMap<usize, Taint>,
        facts: FxHashMap<Span, Taint>,
        sources: &Sources<'_>,
    ) -> Proof {
        let mut proof = Proof::default();
        proof
            .bindings
            .extend(values.iter().filter_map(|(id, value)| {
                (value.derived
                    && value.executable()
                    && !sources.initial.contains(id)
                    && !sources.producers.contains(id))
                .then_some(*id)
            }));
        let mut roots = self.tests.clone();
        let mut candidates = Vec::new();
        for (at, node) in &self.nodes {
            match node {
                Node::Freeze(argument) => roots.push(*argument),
                Node::Invoke(_) => {}
                Node::Ref(id, _)
                    if functions.get(id).is_some_and(Taint::present)
                        || sources.producers.contains(id) =>
                {
                    candidates.push(*at);
                    continue;
                }
                _ => continue,
            }
            if facts.get(at).is_some_and(Taint::present) {
                candidates.push(*at);
            }
        }
        roots.extend(
            self.flows
                .iter()
                .map(|(_, at, _)| *at)
                .chain(self.returns.iter().map(|(_, at)| *at))
                .filter(|at| facts.get(at).is_some_and(|value| value.derived)),
        );
        roots.extend(self.stores.iter().filter_map(|(target, source)| {
            values
                .get(target)
                .is_some_and(|value| value.derived)
                .then_some(*source)
        }));
        let uses = self.project_uses(&facts, sources, &proof.bindings, &mut roots);
        let mut visited = FxHashSet::default();
        while let Some(at) = roots.pop() {
            if !visited.insert(at) {
                continue;
            }
            if let Some(node) = self.nodes.get(&at) {
                proof.transports.insert(at);
                if let Node::Ref(_, at) = node {
                    proof.transports.insert(*at);
                }
                roots.extend(node.children());
            }
        }
        proof.runtime.extend(candidates);
        for (at, callee) in uses {
            if !self.covered(at, callee, sources, &proof) {
                proof.runtime.insert(at);
            }
        }
        proof
    }
    fn covered(&self, at: Span, callee: bool, sources: &Sources<'_>, proof: &Proof) -> bool {
        let mut pending = vec![at];
        let mut seen = FxHashSet::default();
        while let Some(at) = pending.pop() {
            if !seen.insert(at) {
                continue;
            }
            if sources.handled.contains(&at) || proof.transports.contains(&at) {
                continue;
            }
            if sources.candidates.contains(&at)
                || sources.data.contains(&at)
                || proof.runtime.contains(&at)
            {
                return true;
            }
            let Some(node) = self.nodes.get(&at) else {
                continue;
            };
            if !callee && matches!(node, Node::Ref(id, _) if sources.initial.contains(id)) {
                return true;
            }
            pending.extend(if callee {
                match node {
                    Node::Copy(inner) | Node::Read(inner, _) => vec![*inner],
                    _ => Vec::new(),
                }
            } else {
                node.children()
            });
        }
        false
    }
}
fn uncopied(mut at: Span, nodes: &FxHashMap<Span, Node>) -> Span {
    while let Some(Node::Copy(inner)) = nodes.get(&at) {
        at = *inner;
    }
    at
}
fn reference_root(mut at: Span, nodes: &FxHashMap<Span, Node>) -> Option<usize> {
    loop {
        match nodes.get(&at)? {
            Node::Ref(id, _) => return Some(*id),
            Node::Copy(inner) | Node::Read(inner, _) => at = *inner,
            _ => return None,
        }
    }
}
